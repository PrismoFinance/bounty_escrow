use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp, Uint128};

/// Instantiate message to initialize contract state
#[cw_serde]
pub struct InstantiateMsg {
    pub start_bounty_id: u64,
}

/// Message to create a new bounty
#[cw_serde]
pub struct CreateBountyMsg {
    pub title: String,
    pub description: String,
    pub recipient: Option<String>, // Keep as String for now, validated in execute logic
    pub end_height: Option<u64>,
    pub end_time: Option<Timestamp>,
    pub token_denom: String,
    pub quantity: Uint128,
}

/// Message to finalize a bounty
#[cw_serde]
pub struct FinalizeBountyMsg {
    pub bounty_id: u64,
    pub success: bool, // true if successful, false if not
}

/// Message to expire a bounty
#[cw_serde]
pub struct ExpireBountyMsg {
    pub bounty_id: u64,
}

/// Messages for executing contract actions
#[cw_serde]
pub enum ExecuteMsg {
    CreateBounty(CreateBountyMsg),
    FinalizeBounty(FinalizeBountyMsg),
    ExpireBounty(ExpireBountyMsg),
}

/// Query messages for reading contract state
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Fetch a single bounty by ID
    #[returns(BountyResponse)]
    GetBounty(QueryBountyMsg),

    /// List all bounties
    #[returns(Vec<BountyResponse>)]
    ListBounties {},
}

/// Message to query a single bounty
#[cw_serde]
pub struct QueryBountyMsg {
    pub bounty_id: u64,
}

/// Response for a single bounty query
#[cw_serde]
pub struct BountyResponse {
    pub title: String,
    pub description: String,
    pub status: String,
    pub issuer: Addr,
    pub recipient: Option<Addr>,
    pub end_height: Option<u64>,
    pub end_time: Option<Timestamp>,
    pub token_denom: String,
    pub quantity: Uint128,
    pub balance: Uint128,
}
