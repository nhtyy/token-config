use alloy::primitives::Address;
use quote::ToTokens;
use std::collections::HashMap;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct TruthConfig {
    #[serde(flatten)]
    pub chains: HashMap<String, ChainConifg>,
}

#[derive(Debug, Deserialize)]
pub struct ChainConifg {
    pub http_rpc: Option<String>,
    pub ws_rpc: Option<String>,
    pub tokens: Vec<TokenConfig>,
    pub token_groups: Vec<TokenGroup>,
}

#[derive(Debug, Deserialize)]
pub struct TokenGroup {
    pub org: String,
    pub tokens: Vec<TokenConfig>,
}

#[derive(Debug, Deserialize)]
pub struct TokenConfig {
    pub name: UpperCaseTicker,
    #[serde(deserialize_with = "deserialize_address")]
    pub address: Address,
}

fn deserialize_address<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    s.parse().map_err(serde::de::Error::custom)
}

#[derive(Debug)]
pub struct UpperCaseTicker(pub String);

impl ToTokens for UpperCaseTicker {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl<'de> Deserialize<'de> for UpperCaseTicker {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s.chars().all(char::is_uppercase) {
            Ok(UpperCaseTicker(s))
        } else {
            Err(serde::de::Error::custom("expected uppercase token symbol for name"))
        }
    }
}