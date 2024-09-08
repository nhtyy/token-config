//! We want to create a few relationships in the type systems
//! about what addresses are valid for what chains and for what tokens.
//!
//! We want to create a module for each chain
//! Each module will contain
//! - A struct with the name of the chain that implements the Chain trait
//! - A struct for each token that implements the Token trait
//! - An enum of tokens by token group that implements the Token trait

use crate::types::*;

pub fn create_chain_modules(config: TruthConfig) -> proc_macro2::TokenStream {
    let TruthConfig { chains } = config;

    chains
        .into_iter()
        .map(|(name, chain)| create_chain_module(name, chain))
        .collect()
}

pub fn create_chain_module(name: String, chain: ChainConifg) -> proc_macro2::TokenStream {
    let ChainConifg {
        http_rpc,
        ws_rpc,
        tokens,
        token_groups,
    } = chain;

    let http_rpc = if let Some(http_rpc) = http_rpc {
        quote::quote! {
            Some(#http_rpc)
        }
    } else {
        quote::quote! {
            None
        }
    };

    let ws_rpc = if let Some(ws_rpc) = ws_rpc {
        quote::quote! {
            Some(#ws_rpc)
        }
    } else {
        quote::quote! {
            None
        }
    };

    let capitalized_name =
        proc_macro2::Ident::new(&upper_case_first(&name), proc_macro2::Span::call_site());
        
    let mod_name =
        proc_macro2::Ident::new(&name.to_ascii_lowercase(), proc_macro2::Span::call_site());

    let tokens = create_token_structs(capitalized_name.clone(), tokens);
    let token_groups = create_token_groups(capitalized_name.clone(), token_groups);

    quote::quote! {
        pub mod #mod_name {
            pub struct #capitalized_name;

            // note: alloyo doesnt export the URL type, so for now lets just parse at runtime
            impl ::token_config::Chain for #capitalized_name {
                const HTTP_RPC_URL: Option<&'static str> = #http_rpc;

                const WS_RPC_URL: Option<&'static str> = #ws_rpc;
            }

            #tokens
            #token_groups
        }
    }
}

fn create_token_structs(
    chain_name: proc_macro2::Ident,
    tokens: Vec<TokenConfig>,
) -> proc_macro2::TokenStream {
    tokens
        .into_iter()
        .map(|token| {
            let TokenConfig { name, address } = token;

            // ethers has verified the address is valid at compile time in the deser
            let str_address = address.to_string();

            let name = name.0;
            let token_name_ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());

            // Note a ZST is a noop hash
            // however Token is not dyn safe, so we shouldnt run into any unexpectd collisions
            quote::quote! {
                #[derive(Clone, Copy, PartialEq, Eq, Hash)]
                pub struct #token_name_ident;

                impl ::std::fmt::Debug for #token_name_ident {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        write!(f, "{}", #name)
                    }
                }

                impl ::std::fmt::Display for #token_name_ident {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        write!(f, "{}", #name)
                    }
                }

                impl ::token_config::Token for #token_name_ident {
                    type Chain = #chain_name;
                    fn address(&self) -> ::alloy::primitives::Address {
                        static ADDRESS: ::std::sync::LazyLock<::alloy::primitives::Address> = ::std::sync::LazyLock::new(|| {
                            #str_address.parse().expect("failed to parse address")
                        });

                        *ADDRESS
                    }

                    fn symbol(&self) -> ::token_config::UpperCaseSymbol<&str> {
                        ::token_config::UpperCaseSymbol::new_unchecked(#name)
                    }
                }

                impl TryFrom<&::alloy::primitives::Address> for #token_name_ident {
                    type Error = ::token_config::InvalidAddress;
                
                    fn try_from(address: &::alloy::primitives::Address) -> Result<Self, Self::Error> {
                        use ::token_config::Token;

                        if *address == Self.address() {
                            Ok(Self)
                        } else {
                            Err(::token_config::InvalidAddress(std::any::type_name::<Self>()))
                        }
                    }
                }
            }
        })
        .collect()
}

fn create_token_groups(
    chain_name: proc_macro2::Ident,
    token_groups: Vec<TokenGroup>,
) -> proc_macro2::TokenStream {
    token_groups.into_iter().map(|TokenGroup { org, tokens }| {
        let enum_ident = proc_macro2::Ident::new(&upper_case_first(&format!("{}Tokens", org)), proc_macro2::Span::call_site());

        // These will become the actual enum varaiant names
        let names_idents: Vec<proc_macro2::TokenStream> = tokens.iter().map(|TokenConfig { name, .. }| {
            let name = name.0.clone();
            let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());

            quote::quote! {
                #ident
            }
        }).collect();

        let ident_to_name: Vec<proc_macro2::TokenStream> = tokens.iter().map(|TokenConfig { name, .. }| {
            let name = name.0.as_str();
            let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());

            quote::quote! {
                Self::#ident => ::token_config::UpperCaseSymbol::new_unchecked(#name)
            }
        }).collect();

        let ident_to_address: proc_macro2::TokenStream = tokens.iter().map(|TokenConfig { name, address }| {
            let name = name.0.clone();
            let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
            let address = format!("{:?}", address);

            quote::quote! {
                Self::#ident => {
                    static ADDRESS: ::std::sync::LazyLock<::alloy::primitives::Address> = ::std::sync::LazyLock::new(|| {
                        #address.parse().expect("failed to parse address")
                    });
                    *ADDRESS
                }
            }
        }).collect();

        quote::quote! {
            #[derive(Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #enum_ident {
                #(#names_idents),*
            }

            impl ::token_config::Token for #enum_ident {
                type Chain = #chain_name;

                fn address(&self) -> ::alloy::primitives::Address {
                    match self {
                        #ident_to_address
                    }
                }

                fn symbol(&self) -> ::token_config::UpperCaseSymbol<&str> {
                    match self {
                        #(#ident_to_name),*
                    }
                }
            }

            // todo we could not allocate here and just return an array
            impl ::token_config::TokenGroup for #enum_ident {
                const GROUP_NAME: &'static str = stringify!(#org);

                fn all() -> Vec<Self> {
                    vec![#(Self::#names_idents),*]
                }
            }

            impl TryFrom<&::alloy::primitives::Address> for #enum_ident {
                type Error = ::token_config::InvalidAddress;
            
                fn try_from(address: &::alloy::primitives::Address) -> Result<Self, Self::Error> {
                    use ::token_config::{TokenGroup, Token};

                    let token = Self::all()
                        .into_iter()
                        .find(|token| token.address() == *address);
            
                    token.ok_or(::token_config::InvalidAddress(std::any::type_name::<Self>()))
                }
            }
            
            impl ::std::fmt::Debug for #enum_ident {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{}", {
                        let name = match self {
                            #(#ident_to_name),*
                        };

                        format!("{}-{}", #org, name)
                    })
                }
            }

            impl ::std::fmt::Display for #enum_ident {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{}", {
                        let name = match self {
                            #(#ident_to_name),*
                        };

                        format!("{}-{}", #org, name)
                    })
                }
            }
        }
    })
    .collect()
}

fn upper_case_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}
