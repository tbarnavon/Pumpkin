//! Encodes the recipes a 1.21.1 player receives with Storage Drawers installed.
//!
//! 1.21.1 clients build their recipe book from `update_recipes`, so every recipe of the mod that
//! Pumpkin parses must be in it after vanilla's.
#![allow(clippy::expect_used)]

use std::path::Path;

use pumpkin::data::datapack::recipe_loader::parse_recipe;
use pumpkin_data::recipe_sync::SYNCED_RECIPES;
use pumpkin_protocol::java::client::play::encode_before_1_21_2;
use pumpkin_protocol::ser::NetworkReadExt;

#[test]
fn modded_recipes_are_sent_to_1_21_1_clients() {
    let dumps = pumpkin_registry_ext::read_dumps(Path::new(
        "../pumpkin-registry-ext/tests/fixtures/mod-data",
    ))
    .expect("read dumps");
    let mods = pumpkin_registry_ext::install(dumps)
        .expect("install")
        .expect("mods installed");
    let recipes: Vec<_> = mods
        .recipes
        .iter()
        .filter_map(|(id, json)| {
            let (namespace, name) = id.split_once(':')?;
            parse_recipe(namespace, name, &json.to_string())
        })
        .collect();
    // The rest use the mod's own recipe types, crafted by its plugin.
    assert_eq!(recipes.len(), 121);

    let body = encode_before_1_21_2(&recipes).expect("encode");
    let count = (&mut body.as_slice()).get_var_int().expect("count").0;
    assert_eq!(count as usize, SYNCED_RECIPES.len() + recipes.len());
}
