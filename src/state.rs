use std::str::from_utf8;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};

use cosmwasm::errors::{ContractErr, ParseErr, Result, SerializeErr, Unauthorized, Utf8Err};
use cosmwasm::query::perform_raw_query;
use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::storage::Storage;
use cosmwasm::types::{Coin, CosmosMsg, Params, QueryResponse, RawQuery, Response};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Storage, StdResult, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

/// Represents a bounty
#[cw_serde]
pub struct Bounty {
    pub title: String,
    pub description: String,
    pub status: BountyStatus,
    pub issuer: Addr,
    pub recipient: Option<Addr>,
    pub end_height: Option<u64>,
    pub end_time: Option<Timestamp>,
    pub token_denom: String,
    pub quantity: Uint128,
    pub balance: Uint128,
}

/// Status of the bounty
#[cw_serde]
pub enum BountyStatus {
    Open,
    InProgress,
    Completed,
    Expired,
}
/// Map to store all bounties
pub const BOUNTIES: Map<u64, Bounty> = Map::new("bounties");

/// Item to track the next bounty ID
pub const NEXT_BOUNTY_ID: Item<u64> = Item::new("next_bounty_id");

/// Function to return the bounty ID of a specific bounty based on the owner
pub fn bounty_id_by_owner(
    storage: &dyn Storage,
    owner: &Addr,
) -> StdResult<Option<u64>> {
    let mut bounty_ids = BOUNTIES.keys(storage, None, None, cosmwasm_std::Order::Ascending);

    while let Some(id) = bounty_ids.next() {
        let bounty_id = id?;
        let bounty = BOUNTIES.load(storage, bounty_id)?;
        if &bounty.issuer == owner {
            return Ok(Some(bounty_id));
        }

/// Helper to check if a bounty is expired
pub fn check_expired(bounty: &Bounty, env: &Env) -> bool {
    if let Some(end_height) = bounty.end_height {
        if env.block.height > end_height {
            return true;
        }
    }
    if let Some(end_time) = bounty.end_time {
        if env.block.time > end_time {
            return true;
        }
    }
    false
}

/// Helper to validate the bounty status
pub fn ensure_bounty_open(bounty: &Bounty) -> StdResult<()> {
    if bounty.status != BountyStatus::Open {
        Err(StdError::generic_err("Bounty is not open"))
    } else {
        Ok(())
    }
}
    }

    Ok(None)
}
