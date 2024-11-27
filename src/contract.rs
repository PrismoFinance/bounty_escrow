#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Timestamp,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Bounty, BountyStatus, BOUNTIES, NEXT_BOUNTY_ID};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:{{project-name}}";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    NEXT_BOUNTY_ID.save(deps.storage, &1u64)?; // Initialize ID counter
    Ok(Response::new().add_attribute("method", "instantiate").add_attribute("bounty_owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateBounty(msg) => execute_create_bounty(deps, env, info, msg),
        ExecuteMsg::FinalizeBounty(msg) => execute_finalize_bounty(deps, env, info, msg),
        ExecuteMsg::ExpireBounty(msg) => execute_expire_bounty(deps, env, info, msg),
    }
}

/// Create a bounty
pub fn execute_create_bounty(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateBountyMsg,
) -> Result<Response, ContractError> {
    let id = NEXT_BOUNTY_ID.load(deps.storage)?;

    if info.funds.is_empty() || info.funds[0].denom != msg.token_denom {
        return Err(ContractError::InvalidFunds {});
    }
    if info.funds[0].amount < msg.quantity {
        return Err(ContractError::InsufficientFunds {});
    }

    let bounty = Bounty {
        title: msg.title,
        description: msg.description,
        status: BountyStatus::Open,
        issuer: info.sender.clone(),
        recipient: msg.recipient.map(|r| deps.api.addr_validate(&r)).transpose()?,
        end_height: msg.end_height,
        end_time: msg.end_time,
        token_denom: msg.token_denom,
        quantity: msg.quantity,
        balance: info.funds[0].amount,
    };

    BOUNTIES.save(deps.storage, id, &bounty)?;
    NEXT_BOUNTY_ID.save(deps.storage, &(id + 1))?;

    Ok(Response::new()
        .add_attribute("action", "create_bounty")
        .add_attribute("bounty_id", id.to_string())
        .add_attribute("issuer", info.sender.to_string()))
}

/// Finalize a bounty
pub fn execute_finalize_bounty(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: FinalizeBountyMsg,
) -> Result<Response, ContractError> {
    let mut bounty = BOUNTIES.load(deps.storage, msg.bounty_id)?;

    if info.sender != bounty.issuer {
        return Err(ContractError::Unauthorized {});
    }

    if check_expired(&bounty, &env) {
        bounty.status = BountyStatus::Expired;
    }

    if msg.success {
        let recipient = bounty.recipient.ok_or(ContractError::RecipientNotSet {})?;
        bounty.status = BountyStatus::Completed;

        let payment = BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: bounty.token_denom.clone(),
                amount: bounty.balance,
            }],
        };

        BOUNTIES.save(deps.storage, msg.bounty_id, &bounty)?;

        Ok(Response::new()
            .add_message(payment)
            .add_attribute("action", "finalize_bounty")
            .add_attribute("bounty_id", msg.bounty_id.to_string())
            .add_attribute("status", "completed"))
    } else {
        bounty.status = BountyStatus::Expired;

        let refund = BankMsg::Send {
            to_address: bounty.issuer.to_string(),
            amount: vec![Coin {
                denom: bounty.token_denom.clone(),
                amount: bounty.balance,
            }],
        };

        BOUNTIES.save(deps.storage, msg.bounty_id, &bounty)?;

        Ok(Response::new()
            .add_message(refund)
            .add_attribute("action", "finalize_bounty")
            .add_attribute("bounty_id", msg.bounty_id.to_string())
            .add_attribute("status", "expired"))
    }
}

/// Expire a bounty
pub fn execute_expire_bounty(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExpireBountyMsg,
) -> Result<Response, ContractError> {
    let mut bounty = BOUNTIES.load(deps.storage, msg.bounty_id)?;

    if info.sender != bounty.issuer {
        return Err(ContractError::Unauthorized {});
    }

    if !check_expired(&bounty, &env) {
        return Err(ContractError::NotYetExpired {});
    }

    bounty.status = BountyStatus::Expired;

    let refund = BankMsg::Send {
        to_address: bounty.issuer.to_string(),
        amount: vec![Coin {
            denom: bounty.token_denom.clone(),
            amount: bounty.balance,
        }],
    };

    BOUNTIES.save(deps.storage, msg.bounty_id, &bounty)?;

    Ok(Response::new()
        .add_message(refund)
        .add_attribute("action", "expire_bounty")
        .add_attribute("bounty_id", msg.bounty_id.to_string())
        .add_attribute("status", "expired"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBounty { bounty_id } => to_binary(&query_bounty(deps, bounty_id)?),
        QueryMsg::ListBounties {} => to_binary(&query_all_bounties(deps)?),
    }

    pub fn query_bounty(deps: Deps, bounty_id: u64) -> StdResult<Bounty> {
    let bounty = BOUNTIES.load(deps.storage, bounty_id)?;
    Ok(bounty)
}

pub fn query_all_bounties(deps: Deps) -> StdResult<Vec<Bounty>> {
    BOUNTIES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| item.map(|(_, bounty)| bounty))
        .collect()
}

}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        coins, from_binary, testing::{mock_dependencies, mock_env, mock_info}, Addr, Uint128,
    };
    use crate::msg::{CreateBountyMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::{BountyStatus, BOUNTIES, NEXT_BOUNTY_ID};

    /// Helper function to create a test environment with initialized state
    fn setup_contract() -> (DepsMut, Env, MessageInfo) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "token"));

        // Instantiate the contract
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        (deps.as_mut(), env, info)
    }
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &[]); // No funds required for instantiation

    let msg = InstantiateMsg {};
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    assert_eq!(res.attributes, vec![
        ("method", "instantiate"),
        ("bount_owner", "creator"),
    ]);

    // Ensure the NEXT_BOUNTY_ID is initialized to 1
    let id = NEXT_BOUNTY_ID.load(deps.as_ref().storage).unwrap();
    assert_eq!(id, 1u64);
}

#[test]
fn test_create_bounty() {
    let (deps, env, info) = setup_contract();

    let create_msg = ExecuteMsg::CreateBounty(CreateBountyMsg {
        title: "Fix a bug".to_string(),
        description: "Fix a critical bug in the system".to_string(),
        recipient: Some("developer".to_string()),
        end_height: Some(env.block.height + 100),
        end_time: None,
        token_denom: "token".to_string(),
        quantity: Uint128::new(500),
    });

    // Simulate sending the required funds
    let info = mock_info("creator", &coins(500, "token"));
    let res = execute(deps, env.clone(), info.clone(), create_msg).unwrap();

    assert_eq!(res.attributes, vec![
        ("action", "create_bounty"),
        ("bounty_id", "1"),
        ("issuer", "creator"),
    ]);

    // Verify the bounty is stored correctly
    let bounty = BOUNTIES.load(deps.storage, 1).unwrap();
    assert_eq!(bounty.title, "Fix a bug");
    assert_eq!(bounty.description, "Fix a critical bug in the system");
    assert_eq!(bounty.recipient.unwrap(), Addr::unchecked("developer"));
    assert_eq!(bounty.balance, Uint128::new(500));
    assert_eq!(bounty.status, BountyStatus::Open);
}
#[test]
fn test_finalize_bounty_success() {
    let (deps, env, info) = setup_contract();

    // Create a bounty first
    let create_msg = ExecuteMsg::CreateBounty(CreateBountyMsg {
        title: "Fix a bug".to_string(),
        description: "Fix a critical bug in the system".to_string(),
        recipient: Some("developer".to_string()),
        end_height: Some(env.block.height + 100),
        end_time: None,
        token_denom: "token".to_string(),
        quantity: Uint128::new(500),
    });
    let info = mock_info("creator", &coins(500, "token"));
    execute(deps, env.clone(), info.clone(), create_msg).unwrap();

    // Finalize the bounty successfully
    let finalize_msg = ExecuteMsg::FinalizeBounty(FinalizeBountyMsg {
        bounty_id: 1,
        success: true,
    });

    let info = mock_info("creator", &[]);
    let res = execute(deps, env.clone(), info, finalize_msg).unwrap();

    assert_eq!(res.attributes, vec![
        ("action", "finalize_bounty"),
        ("bounty_id", "1"),
        ("status", "completed"),
    ]);

    // Verify the bounty status
    let bounty = BOUNTIES.load(deps.storage, 1).unwrap();
    assert_eq!(bounty.status, BountyStatus::Completed);
}
#[test]
fn test_expire_bounty() {
    let (deps, env, info) = setup_contract();

    // Create a bounty
    let create_msg = ExecuteMsg::CreateBounty(CreateBountyMsg {
        title: "Write documentation".to_string(),
        description: "Write detailed docs for the project".to_string(),
        recipient: None,
        end_height: Some(env.block.height + 1), // Immediate expiration
        end_time: None,
        token_denom: "token".to_string(),
        quantity: Uint128::new(300),
    });
    let info = mock_info("creator", &coins(300, "token"));
    execute(deps, env.clone(), info.clone(), create_msg).unwrap();

    // Advance the block height to simulate expiration
    let mut env = env.clone();
    env.block.height += 10;

    let expire_msg = ExecuteMsg::ExpireBounty(ExpireBountyMsg { bounty_id: 1 });
    let info = mock_info("creator", &[]);
    let res = execute(deps, env, info, expire_msg).unwrap();

    assert_eq!(res.attributes, vec![
        ("action", "expire_bounty"),
        ("bounty_id", "1"),
        ("status", "expired"),
    ]);

    // Verify the bounty status
    let bounty = BOUNTIES.load(deps.storage, 1).unwrap();
    assert_eq!(bounty.status, BountyStatus::Expired);
}
#[test]
fn test_query_all_bounties() {
    let (deps, env, info) = setup_contract();

    // Create multiple bounties
    for i in 1..=3 {
        let create_msg = ExecuteMsg::CreateBounty(CreateBountyMsg {
            title: format!("Bounty {}", i),
            description: "Do something important".to_string(),
            recipient: None,
            end_height: None,
            end_time: None,
            token_denom: "token".to_string(),
            quantity: Uint128::new(100 * i),
        });
        let info = mock_info("creator", &coins(100 * i, "token"));
        execute(deps, env.clone(), info.clone(), create_msg).unwrap();
    }

    let res = query(deps.as_ref(), env, QueryMsg::ListBounties {}).unwrap();
    let bounties: Vec<Bounty> = from_binary(&res).unwrap();

    assert_eq!(bounties.len(), 3);
    assert_eq!(bounties[0].title, "Bounty 1");
    assert_eq!(bounties[1].title, "Bounty 2");
    assert_eq!(bounties[2].title, "Bounty 3");
}

























