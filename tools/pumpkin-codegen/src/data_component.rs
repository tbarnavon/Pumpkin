use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::fs;

/// Generates the `TokenStream` for the `DataComponent` enum and its ID/name conversion methods.
pub fn build() -> TokenStream {
    let mut data_component: BTreeMap<String, u8> =
        serde_json::from_str(&fs::read_to_string("../../assets/data_component.json").unwrap())
            .expect("Failed to parse data_component.json");
    // 1.21.1's components keep their network ids. The components of later versions that the
    // shared game code uses (`consumable`, `equippable`...) follow as internal-only ones: the
    // server derives them from 1.21.1's code and never sends them.
    let networked_count = data_component.len();
    let internal: BTreeMap<String, u8> = serde_json::from_str(
        &fs::read_to_string("../../assets/data_component_internal.json").unwrap(),
    )
    .expect("Failed to parse data_component_internal.json");
    let mut internal: Vec<_> = internal.into_iter().collect();
    internal.sort_by_key(|(_, i)| *i);
    for (name, _) in internal {
        if !data_component.contains_key(&name) {
            let id = u8::try_from(data_component.len()).expect("too many data components");
            data_component.insert(name, id);
        }
    }
    let networked_count = u8::try_from(networked_count).expect("too many data components");

    let mut enum_variants = TokenStream::new();
    let mut id_to_enum = TokenStream::new();
    let mut enum_to_name = TokenStream::new();
    let mut name_to_enum = TokenStream::new();
    let mut data_component_vec = data_component.iter().collect::<Vec<_>>();
    data_component_vec.sort_by_key(|(_, i)| **i);

    for (raw_name, raw_value) in &data_component_vec {
        let strip_name = raw_name
            .strip_prefix("minecraft:")
            .unwrap()
            .replace('/', "_");
        let pascal_case = format_ident!("{}", strip_name.to_pascal_case());

        // Enum variant

        enum_variants.extend(quote! {
            #pascal_case = #raw_value,
        });

        id_to_enum.extend(quote! {
            #raw_value => Some(Self::#pascal_case),
        });

        // TODO use phf
        name_to_enum.extend(quote! {
            #raw_name | #strip_name => Some(Self::#pascal_case),
        });

        // Enum -> &str
        enum_to_name.extend(quote! {
            Self::#pascal_case => #raw_name,
        });
    }

    quote! {
        use crate::data_component_impl::*;

        #[derive(Copy, Clone, Hash, PartialEq, Eq)]
        #[repr(u8)]
        pub enum DataComponent {
            #enum_variants
        }

        impl DataComponent {
            #[must_use]
            pub const fn to_id(self) -> u8 {
                self as u8
            }

            /// Whether 1.21.1 clients know this component; internal-only ones are never sent.
            #[must_use]
            pub const fn is_networked(self) -> bool {
                (self as u8) < #networked_count
            }

            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub const fn try_from_id(id: u8) -> Option<Self> {
                match id {
                    #id_to_enum
                    _ => None,
                }
            }

            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub fn try_from_name(name: &str) -> Option<Self> {
                match name {
                    #name_to_enum
                    _ => None,
                }
            }

            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub const fn to_name(self) -> &'static str {
                match self {
                    #enum_to_name
                }
            }
        }
    }
}
