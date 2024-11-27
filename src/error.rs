use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid funds")]
    InvalidFunds {},

    #[error("Insufficient funds")]
    InsufficientFunds {},

    #[error("Recipient not set")]
    RecipientNotSet {},

    #[error("Bounty already expired")]
    BountyExpired {},

    #[error("Not yet expired")]
    NotYetExpired {},
}
