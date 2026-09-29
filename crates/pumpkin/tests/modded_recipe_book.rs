//! Encodes the `recipe_book_add` packet a player receives with Storage Drawers installed.
//!
//! With `PUMPKIN_PACKET_OUT=<dir>` the packet body is written to `<dir>/recipe_book_add.bin`, so
//! the Extractor's packet check (`-Dpumpkin.checkPackets=<dir>`) can decode it with Minecraft's own
//! codec on a Fabric server running the same mod.
#![allow(clippy::expect_used)]

use std::path::Path;

use pumpkin::data::datapack::recipe_loader::parse_recipe;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::ClientPacket;
use pumpkin_protocol::java::client::play::CRecipeBookAdd;

#[test]
fn modded_recipe_book_encodes() {
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
    assert_eq!(recipes.len(), 127);

    let mut body = Vec::new();
    CRecipeBookAdd::new(true, &recipes)
        .write_packet_data(&mut body, &CURRENT_MC_VERSION)
        .expect("encode");

    if let Ok(dir) = std::env::var("PUMPKIN_PACKET_OUT") {
        std::fs::create_dir_all(&dir).expect("create output dir");
        std::fs::write(Path::new(&dir).join("recipe_book_add.bin"), &body).expect("write packet");
    }
}
