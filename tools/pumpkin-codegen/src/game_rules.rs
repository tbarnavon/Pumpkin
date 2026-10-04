use std::{collections::BTreeMap, fs};

use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde_json::Value;

/// 1.21.1's game rules the game code knows by their later id.
const LATER_NAMES: &[(&str, &str)] = &[
    ("announceAdvancements", "show_advancement_messages"),
    ("commandModificationBlockLimit", "max_block_modifications"),
    ("doDaylightCycle", "advance_time"),
    ("doEntityDrops", "entity_drops"),
    ("doImmediateRespawn", "immediate_respawn"),
    ("doInsomnia", "spawn_phantoms"),
    ("doLimitedCrafting", "limited_crafting"),
    ("doMobLoot", "mob_drops"),
    ("doMobSpawning", "spawn_mobs"),
    ("doPatrolSpawning", "spawn_patrols"),
    ("doTileDrops", "block_drops"),
    ("doTraderSpawning", "spawn_wandering_traders"),
    ("doVinesSpread", "spread_vines"),
    ("doWardenSpawning", "spawn_wardens"),
    ("doWeatherCycle", "advance_weather"),
    ("maxCommandChainLength", "max_command_sequence_length"),
    ("maxCommandForkCount", "max_command_forks"),
    ("naturalRegeneration", "natural_health_regeneration"),
    ("snowAccumulationHeight", "max_snow_accumulation_height"),
    ("spawnRadius", "respawn_radius"),
];

enum InternalRule {
    Bool(bool),
    Int(i64),
}

/// Later versions' rules for what 1.21.1 always does (or does with server properties).
const INTERNAL_RULES: &[(&str, InternalRule)] = &[
    (
        "allow_entering_nether_using_portals",
        InternalRule::Bool(true),
    ),
    ("command_blocks_work", InternalRule::Bool(true)),
    // No limit; `doFireTick` turns fire spread off.
    ("fire_spread_radius_around_player", InternalRule::Int(-1)),
    ("locator_bar", InternalRule::Bool(false)),
    ("player_movement_check", InternalRule::Bool(true)),
    ("pvp", InternalRule::Bool(true)),
    ("spawn_monsters", InternalRule::Bool(true)),
    ("spawner_blocks_work", InternalRule::Bool(true)),
    ("tnt_explodes", InternalRule::Bool(true)),
];

/// Generates the `TokenStream` for the `GameRule` enum, `GameRuleRegistry` struct, and their
/// accessor methods with proper default values for each rule.
pub fn build() -> TokenStream {
    let game_rules: BTreeMap<String, Value> =
        serde_json::from_str(&fs::read_to_string("../../assets/game_rules.json").unwrap())
            .expect("Failed to parse game_rules.json");

    let mut enum_variants = TokenStream::new();
    let mut enum_variants_list = TokenStream::new();
    let mut enum_to_string = TokenStream::new();
    let mut struct_fields = TokenStream::new();
    let mut default_values = TokenStream::new();
    let mut getter_match = TokenStream::new();
    let mut mut_getter_match = TokenStream::new();
    let mut default_functions = TokenStream::new();

    for (raw_name, raw_value) in &game_rules {
        let (variant_type, field_type, default_value) = match raw_value {
            Value::Bool(b) => (quote! { Bool }, quote! { bool }, quote! { #b }),
            Value::Number(n) if n.is_i64() => {
                let i = n.as_i64().unwrap();
                (quote! { Int }, quote! { i64 }, quote! { #i })
            }
            Value::Object(obj) => {
                let default_val = obj
                    .get("default")
                    .expect("Object game rule missing default value");
                match default_val {
                    Value::Number(n) if n.is_i64() => {
                        let i = n.as_i64().unwrap();
                        (quote! { Int }, quote! { i64 }, quote! { #i })
                    }
                    _ => panic!("Unsupported default value type in object for key '{raw_name}'"),
                }
            }
            _ => panic!("Unsupported value type for key '{raw_name}'"),
        };

        // The game code's name for the rule: its later id, where it has one.
        let code_name = LATER_NAMES
            .iter()
            .find(|(id, _)| id == raw_name)
            .map_or(raw_name.as_str(), |(_, later)| later);
        let snake_case = format_ident!("{}", code_name.to_snake_case());
        let pascal_case = format_ident!("{}", code_name.to_pascal_case());
        let default_fn_name = format!("default_{snake_case}");
        let default_fn_ident = format_ident!("default_{snake_case}");

        // Struct field
        struct_fields.extend(quote! {
            #[serde(rename = #raw_name)]
            #[serde(default = #default_fn_name)]
            #[serde(with = "as_string")]
            pub #snake_case: #field_type,
        });

        // Enum variant
        enum_variants.extend(quote! {
            #pascal_case,
        });

        // Enum::all()
        enum_variants_list.extend(quote! {
            Self::#pascal_case,
        });

        // Enum -> &str
        enum_to_string.extend(quote! {
            Self::#pascal_case => write!(f, #raw_name),
        });

        // Default value
        default_values.extend(quote! {
            #snake_case: #default_value,
        });

        // Getter match arms
        getter_match.extend(quote! {
            GameRule::#pascal_case => GameRuleValue::#variant_type(&self.#snake_case),
        });

        mut_getter_match.extend(quote! {
            GameRule::#pascal_case => GameRuleValue::#variant_type(&mut self.#snake_case),
        });

        // Default fn
        default_functions.extend(quote! {
            fn #default_fn_ident() -> #field_type {
                GameRuleRegistry::default().#snake_case
            }
        });
    }

    // Rules of later versions 1.21.1 doesn't have, fixed at their default: not listed by
    // `/gamerule` and not saved.
    for (name, value) in INTERNAL_RULES {
        let snake_case = format_ident!("{name}");
        let default_fn_name = format!("default_{name}");
        let default_fn_ident = format_ident!("default_{name}");
        let (field_type, default_value) = match value {
            InternalRule::Bool(b) => (quote! { bool }, quote! { #b }),
            InternalRule::Int(i) => (quote! { i64 }, quote! { #i }),
        };
        struct_fields.extend(quote! {
            #[serde(skip, default = #default_fn_name)]
            pub #snake_case: #field_type,
        });
        default_values.extend(quote! {
            #snake_case: #default_value,
        });
        default_functions.extend(quote! {
            fn #default_fn_ident() -> #field_type {
                #default_value
            }
        });
    }

    quote! {
        use std::fmt;
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum GameRule {
            #enum_variants
        }

        impl GameRule {
            pub const fn all() -> &'static [Self] {
                &[
                    #enum_variants_list
                ]
            }
        }

        impl fmt::Display for GameRule {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    #enum_to_string
                }
            }
        }

        #[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
        pub struct GameRuleRegistry {
            #struct_fields
        }

        pub enum GameRuleValue<I, B> {
            Int(I),
            Bool(B),
        }

        impl<I: fmt::Display, B: fmt::Display> fmt::Display for GameRuleValue<I, B> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    Self::Int(v) => write!(f, "{v}"),
                    Self::Bool(v) => write!(f, "{v}"),
                }
            }
        }

        impl GameRuleRegistry {
            pub fn get(&self, rule: &GameRule) -> GameRuleValue<&i64, &bool> {
                match rule {
                    #getter_match
                }
            }

            pub fn get_mut(&mut self, rule: &GameRule) -> GameRuleValue<&mut i64, &mut bool> {
                match rule {
                    #mut_getter_match
                }
            }
        }

        impl Default for GameRuleRegistry {
            fn default() -> Self {
                Self {
                    #default_values
                }
            }
        }

        #default_functions

        mod as_string {
            use serde::{Serialize, Deserialize, Serializer, Deserializer};
            use std::{fmt::Display, str::FromStr};

            pub fn serialize<T: Display, S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&value.to_string())
            }

            pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
            where
                T: FromStr,
                D: Deserializer<'de>,
                <T as FromStr>::Err: Display,
            {
                let s = String::deserialize(deserializer)?;
                s.parse::<T>().map_err(serde::de::Error::custom)
            }
        }
    }
}
