use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub arbiter: String, // This will be an arbiter that approves or denies the bounty completion. 
    pub recipient: String, // This will be 'destinations' from the bounty contract 
    // you can set a last time or block height the contract is valid at
    // if *either* is non-zero and below current state, the contract is considered expired
    // and will be returned to the original funder
    pub end_height: i64,
    pub end_time: i64,
}

#[cw_serde]
pub enum ExecuteMsg {
    ApproveEscrow {
        // release some coins - if quantity is None, release all coins in balance
        quantity: Option<Vec<Coin>>,
        recipient: String,
    },
    RefundEscrow {
        owner: String,
        quantity: Option<Vec<Coin>>,
    },
    ExpiredEscrow {
        owner: String,
        quantity: Option<Vec<Coin>>,
        end_height: i64,
        end_time: i64,
    },
    UpdateEscrow {
    bounty_owner: Option<String>, // This is the Bounty Issuer of the bounty contract. 
    arbiter: Option<String>, 
    recipient: Option<String>, // This is destinations
    quantity: Uint128, 
    token_denom: Uint128, 
    },
}


#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetEscrowResponse)]
    GetEscrow {escrow_id},
    #[returns(GetEscrowRecipient)]
    GetRecipient {recipient},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetEscrowResponse {
    pub status: BountyStatus, // This will be the bounty status
    pub recipient: String, // Who will receive the funds in escrow? 1) Rejection --> Bounty Issuer 2) Acceptance --> Bounty Assignee
    pub arbiter: String, 
    pub pay_amount: Uint128, 
}

#[cw_serde]
pub struct GetEscrowRecipient {
    pub recipient: String,
}

