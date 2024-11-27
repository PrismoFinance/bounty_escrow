use std::str::from_utf8;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};

use cosmwasm::errors::{ContractErr, ParseErr, Result, SerializeErr, Unauthorized, Utf8Err};
use cosmwasm::query::perform_raw_query;
use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::storage::Storage;
use cosmwasm::types::{Coin, CosmosMsg, Params, QueryResponse, RawQuery, Response};

impl GenericBalance {
    pub fn add_tokens(&mut self, add: Balance) {
        match add {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    match index {
                        Some(idx) => self.native[idx].amount += token.amount,
                        None => self.native.push(token),
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                match index {
                    Some(idx) => self.cw20[idx].amount += token.amount,
                    None => self.cw20.push(token),
                }
            }
        };
    }
}


// Bounty Issuer will act as the Arbiter
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

#[cw_serde] 
pub enum BountyStatus {
    Open,
    InProgress,
    Completed,
    Expired,
}

impl Bounty {
    pub fn is_expired(&self, env: &Env) -> bool {
        if let Some(end_height) = self.end_height {
            if env.block.height > end_height {
                return true;
            }
        }

        if let Some(end_time) = self.end_time {
            if env.block.time > Timestamp::from_seconds(end_time) {
                return true;
            }
        }

        false
    }

    pub fn human_whitelist(&self) -> Vec<String> {
        self.cw20_whitelist.iter().map(|a| a.to_string()).collect()
    }
}

pub const BOUNTIES: Map<&str, Bounty> = Map::new("bounties");

// Function to return the escrow_id of a specific escrow based on owner
pub fn bounty_id_by_owner(
    storage: &dyn Storage,
    owner: &Addr,            // Updated to search by 'owner'
) -> StdResult<String> {
    let bounty_id: Vec<String> = BOUNTIES
        .keys(storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    // Search for the escrow created by the given owner
    for id in bounty_id {
        let bounty = BOUNTIES.load(storage, &bounty_id)?;
        if &bounty.owner == owner { // Updated from 'creator' to 'owner'
            return Ok(bounty_id); // Return the id of the escrow created by this owner
        }
    }


