use std::fs;

use proc_macro2::TokenStream;
use quote::quote;

/// Generates the network order of the `command_argument_type` registry.
pub fn build() -> TokenStream {
    let types: Vec<String> = serde_json::from_str(
        &fs::read_to_string("../../assets/command_argument_types.json").unwrap(),
    )
    .expect("Failed to parse command_argument_types.json");

    quote! {
        /// Command argument type names, indexed by their network id.
        pub const COMMAND_ARGUMENT_TYPES: &[&str] = &[#(#types),*];

        /// Returns the network id of the command argument type `name`, if this version has it.
        #[must_use]
        pub fn command_argument_type_id(name: &str) -> Option<i32> {
            COMMAND_ARGUMENT_TYPES.iter().position(|t| *t == name).map(|i| i as i32)
        }
    }
}
