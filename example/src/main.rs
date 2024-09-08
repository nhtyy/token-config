token_config::from_json_config!("truth.base.example.json");

use base::UniversalTokens;
use token_config::{Token, TokenGroup};

fn main() {
    for token in UniversalTokens::all() {
        println!("{:?}", token);
        println!("{}", token.address());
        println!("{}", token.symbol());
    }   
}