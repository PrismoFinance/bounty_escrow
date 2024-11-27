use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct CreateBountyMsg {
    pub title: String,
    pub description: String,
    pub recipient: Option<String>,
    pub end_height: Option<u64>,
    pub end_time: Option<cosmwasm_std::Timestamp>,
    pub token_denom: String,
    pub quantity: cosmwasm_std::Uint128,
}

#[cw_serde]
pub struct FinalizeBountyMsg {
    pub bounty_id: u64,
    pub success: bool,
}

#[cw_serde]
pub struct ExpireBountyMsg {
    pub bounty_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateBounty(CreateBountyMsg),
    FinalizeBounty(FinalizeBountyMsg),
    ExpireBounty(ExpireBountyMsg),
}



#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    GetBounty(QueryBountyMsg),
    ListBounties {}, // Fetch all bounties
}

#[cw_serde]
pub struct QueryBountyMsg {
    pub bounty_id: u64,
}

