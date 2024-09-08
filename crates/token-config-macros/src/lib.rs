use proc_macro::TokenStream;
use std::path::PathBuf;
use syn::{Error, LitStr};

mod chains;
mod types;
use types::*;

#[proc_macro]
pub fn from_json_config(input: TokenStream) -> TokenStream {
    let config = match get_config_from_file(input) {
        Ok(config) => config,
        Err(e) => return e.to_compile_error().into(),
    };

    chains::create_chain_modules(config).into()
}

fn get_config_from_file(input: TokenStream) -> syn::Result<TruthConfig> {
    let lit = syn::parse::<LitStr>(input)?;
    let span = lit.span();

    let mut path = PathBuf::from(lit.value());
    if path.is_relative() {
        let manifest = std::env::var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| Error::new(span, "failed to get manifest dir"))?;

        path = manifest.join(path);
    }

    path = path
        .canonicalize()
        .map_err(|e| Error::new(span, format!("failed to canon path to truth config {e}")))?;

    let config_str = std::fs::read_to_string(&path)
        .map_err(|e| Error::new(span, format!("Failed to read truth config to string {e}")))?;

    Ok(serde_json::from_str(&config_str)
        .map_err(|e| Error::new(span, format!("Failed to desererialize truth config: {e}")))?)
}
