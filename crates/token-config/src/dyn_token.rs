use std::{
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};

use alloy::primitives::Address;

use crate::{Chain, Token, UpperCaseSymbol};

pub struct DynToken<C> {
    symbol: UpperCaseSymbol<String>,
    address: Address,
    display_str: String,
    _chain: PhantomData<C>,
}

impl<C> DynToken<C> {
    pub fn new(symbol: UpperCaseSymbol<String>, address: Address, display_str: String) -> Self {
        Self {
            symbol,
            address,
            display_str,
            _chain: PhantomData,
        }
    }

    pub fn from_token<T>(token: T) -> Self
    where
        T: Token<Chain = C>,
    {
        Self {
            symbol: token.symbol().owned(),
            address: token.address(),
            display_str: token.to_string(),
            _chain: PhantomData,
        }
    }
}

impl<C> Clone for DynToken<C> {
    fn clone(&self) -> Self {
        Self {
            symbol: self.symbol.clone(),
            address: self.address,
            display_str: self.display_str.clone(),
            _chain: PhantomData,
        }
    }
}

impl<C> PartialEq for DynToken<C> {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl<C> Eq for DynToken<C> {}

impl<C> Hash for DynToken<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.display_str.hash(state);
        std::any::type_name::<C>().hash(state);
    }
}

impl<C> Display for DynToken<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_str)
    }
}

impl<C> Debug for DynToken<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_str)
    }
}

impl<C: Chain> Token for DynToken<C> {
    type Chain = C;

    fn address(&self) -> Address {
        self.address
    }

    fn symbol(&self) -> UpperCaseSymbol<&str> {
        self.symbol.as_str()
    }
}
