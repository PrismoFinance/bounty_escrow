use std::str::from_utf8;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};

use cosmwasm::errors::{ContractErr, ParseErr, Result, SerializeErr, Unauthorized, Utf8Err};
use cosmwasm::query::perform_raw_query;
use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::storage::Storage;
use cosmwasm::types::{Coin, CosmosMsg, Params, QueryResponse, RawQuery, Response};


#[cw_serde]
pub struct EscrowBounty {
    bounty_owner: Addr, // This is the Bounty Issuer of the bounty contract. 
    arbiter: Addr, 
    recipient: Addr, // This is destinations
    quantity: Uint128, 
    token_denom: Uint128, 
}

#[cw_serde] 
pub enum EscrowStatus {
    Completed {recipient: Addr, quantity: Uint128, token_denom: Uint128},
    Rejected {bounty_owner: Addr, quantity: Uint128, token_denom: Uint128}, 
    In Progress {}, 
    Expired {}, 
}

