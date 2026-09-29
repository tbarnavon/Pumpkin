//! Installs the real Storage Drawers 26.3.0.1 dump (made by the Extractor's mod dump on a Fabric
//! 26.3 server) and checks Pumpkin ends up with the same ids that server assigned.
#![allow(clippy::expect_used)]

use std::path::Path;

use pumpkin_data::data_component::DataComponent;
use pumpkin_data::dynamic::names::{SyncedRegistry, modded_id};
use pumpkin_data::item::Item;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockId, BlockStateId};

#[test]
fn storage_drawers_dump_installs_with_reference_ids() {
    let dumps =
        pumpkin_registry_ext::read_dumps(Path::new("tests/fixtures/mod-data")).expect("read");
    assert_eq!(dumps.len(), 1);
    // `install` itself fails if any id differs from the raw ids recorded in the dump.
    let installed = pumpkin_registry_ext::install(dumps)
        .expect("install")
        .expect("mods installed");
    assert!(std::ptr::eq(
        installed,
        pumpkin_registry_ext::installed().expect("installed")
    ));
    assert_eq!(installed.namespaces, ["storagedrawers"]);
    assert_eq!(installed.recipes.len(), 132);
    assert_eq!(installed.loot_tables.len(), 15);

    assert_eq!(BlockId::count(), BlockId::VANILLA_COUNT + 150);
    assert_eq!(BlockStateId::count(), BlockStateId::VANILLA_COUNT + 965);
    assert_eq!(Item::count(), Item::VANILLA_COUNT + 166);

    let drawer = Block::from_name("storagedrawers:oak_full_drawers_1").expect("block");
    assert_eq!(drawer.id.as_u16(), BlockId::VANILLA_COUNT);
    assert_eq!(drawer.states.len(), 4);
    let item = Item::from_registry_key("storagedrawers:oak_full_drawers_1").expect("item");
    assert_eq!(drawer.item_id, item.id);
    // Placing the item places the block.
    assert_eq!(Block::from_item_id(item.id).map(|b| b.id), Some(drawer.id));
    assert_eq!(
        drawer.default_state.block_entity_type,
        modded_id(
            SyncedRegistry::BlockEntityType,
            "storagedrawers:standard_drawers_1"
        )
        .expect("be type")
    );
    let south = drawer
        .state_from_properties(&[("facing", "south")])
        .expect("state");
    assert_eq!(south.id.as_u16(), BlockStateId::VANILLA_COUNT + 1);

    // Mods add their blocks to vanilla tags; mining speed depends on this.
    assert!(drawer.has_tag(&tag::Block::MINECRAFT_MINEABLE_AXE));

    // Every modded item got at least its stack size from the dumped default components.
    for raw in Item::VANILLA_COUNT..Item::count() {
        let item = Item::from_id(raw).expect("modded item");
        assert!(
            item.components
                .iter()
                .any(|(kind, _)| *kind == DataComponent::MaxStackSize),
            "{} has no max_stack_size",
            item.registry_key
        );
    }
}
