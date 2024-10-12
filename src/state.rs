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

#[cw_serde]
pub struct EscrowBounty {
    pub bounty_owner: Addr, // This is the Bounty Issuer of the bounty contract. 
    pub arbiter: Addr, 
    pub status: EscrowStatus,
    pub recipient: Addr, // This is destinations
    pub quantity: Uint128, 
    pub token_denom: Uint128, 
    /// Title of the escrow, for example for a bug bounty "Fix issue in contract.rs"
    pub title: String,
    /// Description of the escrow, a more in depth description of how to meet the escrow condition
    pub description: String,
    /// When end height set and block height exceeds this value, the escrow is expired.
    /// Once an escrow is expired, it can be returned to the original funder (via "refund").
    pub end_height: Option<u64>,
    /// When end time (in seconds since epoch 00:00:00 UTC on 1 January 1970) is set and
    /// block time exceeds this value, the escrow is expired.
    /// Once an escrow is expired, it can be returned to the original funder (via "refund").
    pub end_time: Option<u64>,
      /// Balance in Native and Cw20 tokens
    pub balance: GenericBalance,
    /// All possible contracts that we accept tokens from
    pub cw20_whitelist: Vec<Addr>,
}

impl EscrowBounty {
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

pub const ESCROWS: Map<&str, Escrow> = Map::new("escrow");

/// This returns the list of ids for all registered escrows
pub fn all_escrow_ids(storage: &dyn Storage) -> StdResult<Vec<String>> {
    ESCROWS
        .keys(storage, None, None, Order::Ascending)
        .collect()
}

#[cw_serde]
pub struct EscrowBounty {
    bounty_owner: Addr, // This is the Bounty Issuer of the bounty contract. 
    arbiter: Addr, 
    recipient: Addr, // This is destinations
    quantity: Uint128, 
    token_denom: Uint128, 
    /// Title of the escrow, for example for a bug bounty "Fix issue in contract.rs"
    pub title: String,
    /// Description of the escrow, a more in depth description of how to meet the escrow condition
    pub description: String,
    /// When end height set and block height exceeds this value, the escrow is expired.
    /// Once an escrow is expired, it can be returned to the original funder (via "refund").
    pub end_height: Option<u64>,
    /// When end time (in seconds since epoch 00:00:00 UTC on 1 January 1970) is set and
    /// block time exceeds this value, the escrow is expired.
    /// Once an escrow is expired, it can be returned to the original funder (via "refund").
    pub end_time: Option<u64>,
      /// Balance in Native and Cw20 tokens
    pub balance: GenericBalance,
    /// All possible contracts that we accept tokens from
    pub cw20_whitelist: Vec<Addr>,
}

#[cw_serde] 
pub enum EscrowStatus {
    Completed {recipient: Addr, quantity: Uint128, token_denom: Uint128},
    Rejected {bounty_owner: Addr, quantity: Uint128, token_denom: Uint128}, 
    In Progress {}, 
    Expired {}, 
}

