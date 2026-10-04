use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use syn::LitInt;

/// Raw deserialization shape for a single attribute entry from `attributes.json`.
#[derive(Deserialize)]
struct Attributes {
    /// Numeric registry ID for this attribute.
    id: u8,
    /// Default numeric value applied to entities that do not override this attribute.
    default_value: f64,
}

/// The constant for an attribute name. 1.21.1 prefixes attributes (`generic.armor`,
/// `player.block_break_speed`); the constant drops the prefix (`ARMOR`), as later versions do.
pub(crate) fn attribute_ident(name: &str) -> proc_macro2::Ident {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    let name = name.rsplit('.').next().unwrap_or(name);
    format_ident!("{}", name.to_shouty_snake_case())
}

/// Generates the `TokenStream` for the `Attributes` struct and its associated constants.
pub fn build() -> TokenStream {
    let attributes: BTreeMap<String, Attributes> =
        serde_json::from_str(&fs::read_to_string("../../assets/attributes.json").unwrap())
            .expect("Failed to parse attributes.json");

    let mut sorted_attributes: Vec<(String, Attributes)> = attributes.into_iter().collect();
    sorted_attributes.sort_by_key(|(_, raw)| raw.id);

    let mut constant_defs = Vec::new();
    let mut constant_idents = Vec::new();

    for (raw_name, raw_value) in sorted_attributes {
        let constant_ident = attribute_ident(&raw_name);
        constant_idents.push(constant_ident.clone());

        let id_lit = LitInt::new(&raw_value.id.to_string(), Span::call_site());
        let default_value_lit = raw_value.default_value;
        let name_str = format!("minecraft:{raw_name}");

        constant_defs.push(quote!(
            pub const #constant_ident: Self = Self {
                id: #id_lit,
                default_value: #default_value_lit,
                name: #name_str,
            };
        ));
    }

    quote! {
        use std::hash::Hash;

        #[derive(Clone, Debug)]
        pub struct Attributes {
            pub id: u8,
            pub default_value: f64,
            pub name: &'static str,
        }
        impl PartialEq for Attributes {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }
        impl Eq for Attributes {}
        impl Hash for Attributes {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }
        impl Attributes {
            #(#constant_defs)*

            pub const ALL: &'static [Self] = &[
                #(Self::#constant_idents),*
            ];
        }
    }
}
