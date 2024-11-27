use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Timestamp,
};
use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Bounty, BOUNTIES, NEXT_BOUNTY_ID, BountyStatus};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:{{project-name}}";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    NEXT_BOUNTY_ID.save(deps.storage, &msg.start_bounty_id)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("bounty_owner", info.sender))
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
pub mod execute {
    use super::*;
// Place functions herepub fn create_bounty(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateBountyMsg,
) -> Result<Response, ContractError> {
    // Get the next bounty ID
    let id = NEXT_BOUNTY_ID.load(deps.storage)?;

    // Validate that the funds sent match the bounty requirements
    if info.funds.len() != 1 || info.funds[0].denom != msg.token_denom {
        return Err(ContractError::InvalidFunds {});
    }
    if info.funds[0].amount < msg.quantity {
        return Err(ContractError::InsufficientFunds {});
    }

    if info.funds.is_empty() || info.funds[0].denom != msg.token_denom {
    return Err(ContractError::InvalidFunds {});
}

    // Create the bounty
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

    // Save the bounty in storage
    BOUNTIES.save(deps.storage, id, &bounty)?;

    // Increment the next bounty ID
    NEXT_BOUNTY_ID.save(deps.storage, &(id + 1))?;

    Ok(Response::new()
        .add_attribute("action", "create_bounty")
        .add_attribute("bounty_id", id.to_string())
        .add_attribute("issuer", info.sender.to_string()))
}


        pub fn finalize_bounty(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: FinalizeBountyMsg,
) -> Result<Response, ContractError> {
    // Load the bounty
    let mut bounty = BOUNTIES.load(deps.storage, msg.bounty_id)?;

    // Ensure the caller is the bounty issuer
    if info.sender != bounty.issuer {
        return Err(ContractError::Unauthorized {});
    }

    // Check if the bounty has expired
    if let Some(end_height) = bounty.end_height {
        if env.block.height > end_height {
            bounty.status = BountyStatus::Expired;
        }
    }
    if let Some(end_time) = bounty.end_time {
        if env.block.time > end_time {
            bounty.status = BountyStatus::Expired;
        }
    }

    // Handle completion or failure
    if msg.success {
        // Ensure a recipient is set
        let recipient = bounty.recipient.ok_or(ContractError::RecipientNotSet {})?;

        // Mark bounty as completed
        bounty.status = BountyStatus::Completed;

        // Create a bank message to transfer funds to the recipient
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
        // Mark bounty as expired and refund the issuer
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

pub fn expire_bounty(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExpireBountyMsg,
) -> Result<Response, ContractError> {
    // Load the bounty
    let mut bounty = BOUNTIES.load(deps.storage, msg.bounty_id)?;

    // Ensure the caller is the bounty issuer
    if info.sender != bounty.issuer {
        return Err(ContractError::Unauthorized {});
    }

    // Check if the bounty has already expired
    if let Some(end_height) = bounty.end_height {
        if env.block.height <= end_height {
            return Err(ContractError::NotYetExpired {});
        }
    }
    if let Some(end_time) = bounty.end_time {
        if env.block.time <= end_time {
            return Err(ContractError::NotYetExpired {});
        }
    }

    // Mark the bounty as expired
    bounty.status = BountyStatus::Expired;

    // Refund the funds to the issuer
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

        
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBounty { bounty_id } => to_binary(&query_bounty(deps, bounty_id)?),
        QueryMsg::ListBounties {} => to_binary(&query_all_bounties(deps)?),
    }
}





















// Ignore for now
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
