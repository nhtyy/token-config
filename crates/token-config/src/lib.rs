//! The Truth crate encodes the relationships between token addresses, "token groups", and chains into the type system
//!
//! "Token Groups" tie a tokens tokens togethor by organization.
//! For example you may create an orignation to represent a set of wrapped assets from a certian custodian.
//!
//! The truth can enforce things like at compile time
//! - If two tokens are on the same chain
//! - If two tokens are in the same token group
//! - If a token is in a token group
//! - If a token is on a chain
//!
//!
//! Example config:
#![doc = include_str!("../../../truth.example.json")]

use thiserror::Error;

use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Deref;

pub mod dyn_token;

pub use alloy::primitives::Address;

#[rustfmt::skip]
pub trait Token:
    Clone 
    + Hash 
    + Eq 
    + Sized 
    + Debug 
    + Display 
    + Send 
    + Sync
{
    type Chain: Chain;

    fn address(&self) -> Address;

    fn symbol(&self) -> UpperCaseSymbol<&str>;
}

pub trait Chain: Send + Sync {
    const HTTP_RPC_URL: Option<&'static str>;

    const WS_RPC_URL: Option<&'static str>;
}

pub trait TokenGroup: Token {
    const GROUP_NAME: &'static str;

    fn all() -> Vec<Self>;
}

#[derive(Debug, Error)]
#[error("Not a valid address for this type. {}", .0)]
pub struct InvalidAddress(pub &'static str);

#[derive(Debug, Clone)]
pub struct UpperCaseSymbol<T>(T);

impl<T> Display for UpperCaseSymbol<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl UpperCaseSymbol<String> {
    pub fn as_str(&self) -> UpperCaseSymbol<&str> {
        UpperCaseSymbol(self.0.as_str())
    }
}

impl UpperCaseSymbol<&str> {
    pub fn owned(&self) -> UpperCaseSymbol<String> {
        UpperCaseSymbol(self.0.to_owned())
    }
}

impl<T> UpperCaseSymbol<T> {
    pub fn new_unchecked(symbol: T) -> Self {
        UpperCaseSymbol(symbol)
    }
}

impl<T> Deref for UpperCaseSymbol<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub use token_config_macros::from_json_config;
