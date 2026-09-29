//! Installing modded registry entries must leave every vanilla id where it was and number modded
//! entries directly after vanilla, in insertion order.
//!
//! The overlay is process-global and can only be frozen once, so everything runs in one test.
#![allow(
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::missing_const_for_fn
)]

use pumpkin_data::block_properties::NoteblockInstrument;
use pumpkin_data::block_state::PistonBehavior;
use pumpkin_data::dynamic::DynamicRegistriesBuilder;
use pumpkin_data::dynamic::FreezeError;
use pumpkin_data::dynamic::blocks::{BlockDef, StateDef};
use pumpkin_data::dynamic::items::ItemDef;
use pumpkin_data::dynamic::names::{SyncedRegistry, modded_id, modded_name};
use pumpkin_data::dynamic::tags::TagDef;
use pumpkin_data::item::Item;
use pumpkin_data::tag::{self, RegistryKey, Taggable};
use pumpkin_data::{Block, BlockId, BlockState, BlockStateId};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

fn full_cube() -> BoundingBox {
    BoundingBox::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0))
}

fn slab_shape() -> BoundingBox {
    // Not in the vanilla table with these exact bounds, so it gets a new shape index.
    BoundingBox::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.3125, 1.0))
}

fn state(values: &[&str], block_entity: Option<&str>) -> StateDef {
    StateDef {
        values: values.iter().map(|v| (*v).to_string()).collect(),
        state_flags: 0,
        side_flags: 0,
        instrument: NoteblockInstrument::Harp,
        luminance: 0,
        piston_behavior: PistonBehavior::Block,
        hardness: 3.0,
        opacity: 15,
        collision_shapes: vec![full_cube()],
        outline_shapes: vec![full_cube()],
        block_entity_type: block_entity.map(str::to_string),
        random_ticks: false,
    }
}

fn block(
    name: &str,
    properties: Vec<(&str, Vec<&str>)>,
    states: Vec<StateDef>,
    default_state: usize,
) -> BlockDef {
    BlockDef {
        name: name.to_string(),
        hardness: 3.0,
        blast_resistance: 3.0,
        map_color: 0,
        slipperiness: 0.6,
        velocity_multiplier: 1.0,
        jump_velocity_multiplier: 1.0,
        item_id: 0,
        flammable: None,
        experience: None,
        properties: properties
            .into_iter()
            .map(|(name, values)| {
                (
                    name.to_string(),
                    values.into_iter().map(str::to_string).collect(),
                )
            })
            .collect(),
        states,
        default_state,
    }
}

#[test]
fn modded_entries_follow_vanilla_and_leave_it_untouched() {
    // Snapshot every vanilla state -> block mapping and a few names before installing mods.
    let vanilla_states: Vec<(u16, u16)> = (0..BlockStateId::VANILLA_COUNT)
        .map(|raw| {
            let id = BlockStateId::new(raw).expect("vanilla state");
            (raw, Block::from_state_id(id).id.as_u16())
        })
        .collect();
    let stone_default = Block::STONE.default_state.id;
    let vanilla_item_count = Item::count();
    assert_eq!(BlockStateId::new(BlockStateId::VANILLA_COUNT), None);
    assert_eq!(BlockId::count(), BlockId::VANILLA_COUNT);

    // A builder that fails validation must not install anything.
    let mut bad = DynamicRegistriesBuilder::new();
    bad.add_block(block(
        "minecraft:not_allowed",
        vec![],
        vec![state(&[], None)],
        0,
    ));
    assert!(matches!(bad.freeze(), Err(FreezeError::InvalidName { .. })));
    assert_eq!(BlockId::count(), BlockId::VANILLA_COUNT);

    let mut builder = DynamicRegistriesBuilder::new();
    builder.add_entry(SyncedRegistry::BlockEntityType, "testmod:drawer");
    builder.add_item(ItemDef {
        name: "testmod:drawer".to_string(),
        components: Vec::new(),
    });
    // Two properties; states listed in the order the mod's StateDefinition yields them.
    let mut drawer_states = Vec::new();
    for facing in ["north", "south", "west", "east"] {
        for open in ["true", "false"] {
            drawer_states.push(state(&[facing, open], Some("testmod:drawer")));
        }
    }
    builder.add_block(block(
        "testmod:drawer",
        vec![
            ("facing", vec!["north", "south", "west", "east"]),
            ("open", vec!["true", "false"]),
        ],
        drawer_states,
        1,
    ));
    let mut slab = state(&[], None);
    slab.collision_shapes = vec![slab_shape()];
    slab.random_ticks = true;
    builder.add_block(block("testmod:trim", vec![], vec![slab], 0));
    builder.add_tag_entries(TagDef {
        registry: RegistryKey::Block,
        name: "minecraft:mineable/axe".to_string(),
        entries: vec!["testmod:drawer".to_string()],
    });
    builder.add_tag_entries(TagDef {
        registry: RegistryKey::Block,
        name: "testmod:storage".to_string(),
        entries: vec![
            "testmod:drawer".to_string(),
            "minecraft:chest".to_string(),
            "#minecraft:logs".to_string(),
        ],
    });
    builder.freeze().expect("freeze");

    // Vanilla ids are unchanged.
    for (raw, block) in &vanilla_states {
        let id = BlockStateId::new(*raw).expect("vanilla state");
        assert_eq!(
            Block::from_state_id(id).id.as_u16(),
            *block,
            "state {raw} moved"
        );
    }
    assert_eq!(Block::STONE.default_state.id, stone_default);
    assert_eq!(
        Block::from_name("minecraft:stone").map(|b| b.id),
        Some(BlockId::STONE)
    );
    assert_eq!(
        Block::from_registry_key("stone").map(|b| b.id),
        Some(BlockId::STONE)
    );
    assert!(Block::STONE.is_vanilla());

    // Modded blocks come right after vanilla, in insertion order.
    let drawer = Block::from_name("testmod:drawer").expect("drawer registered");
    let trim = Block::from_registry_key("testmod:trim").expect("trim registered");
    assert_eq!(drawer.id.as_u16(), BlockId::VANILLA_COUNT);
    assert_eq!(trim.id.as_u16(), BlockId::VANILLA_COUNT + 1);
    assert_eq!(BlockId::count(), BlockId::VANILLA_COUNT + 2);
    assert_eq!(drawer.namespaced_name(), "testmod:drawer");
    assert!(!drawer.is_vanilla());

    // States are numbered consecutively after vanilla: 8 drawer states, then the trim.
    assert_eq!(drawer.states.len(), 8);
    for (offset, state) in drawer.states.iter().enumerate() {
        assert_eq!(
            state.id.as_u16(),
            BlockStateId::VANILLA_COUNT + offset as u16
        );
        assert_eq!(Block::from_state_id(state.id).id, drawer.id);
        assert!(std::ptr::eq(BlockState::from_id(state.id), state));
    }
    assert_eq!(
        trim.default_state.id.as_u16(),
        BlockStateId::VANILLA_COUNT + 8
    );
    assert_eq!(BlockStateId::count(), BlockStateId::VANILLA_COUNT + 9);
    assert!(BlockStateId::new(BlockStateId::VANILLA_COUNT + 8).is_some());
    assert_eq!(BlockStateId::new(BlockStateId::VANILLA_COUNT + 9), None);

    // Default state and properties.
    assert_eq!(
        drawer.default_state.id.as_u16(),
        BlockStateId::VANILLA_COUNT + 1
    );
    let props = drawer
        .properties(drawer.default_state.id)
        .expect("drawer has properties");
    assert_eq!(
        props.to_props(),
        vec![("facing", "north"), ("open", "false")]
    );
    let west_open = drawer.from_properties(&[("facing", "west"), ("open", "true")]);
    assert_eq!(
        west_open.to_state_id(drawer).as_u16(),
        BlockStateId::VANILLA_COUNT + 4
    );
    // Missing properties keep the default state's values.
    let east = drawer.from_properties(&[("facing", "east")]);
    assert_eq!(east.to_props(), vec![("facing", "east"), ("open", "false")]);
    let found = drawer
        .state_from_properties(&[("facing", "south"), ("open", "true")])
        .expect("state exists");
    assert_eq!(found.id.as_u16(), BlockStateId::VANILLA_COUNT + 2);
    assert!(trim.properties(trim.default_state.id).is_none());

    // Block entity type resolves to the modded entry.
    let drawer_be = modded_id(SyncedRegistry::BlockEntityType, "testmod:drawer").expect("be type");
    assert_eq!(drawer_be, SyncedRegistry::BlockEntityType.vanilla_len());
    assert_eq!(drawer.default_state.block_entity_type, drawer_be);
    assert_eq!(
        modded_name(SyncedRegistry::BlockEntityType, drawer_be),
        Some("testmod:drawer")
    );

    // Shapes: the full cube reuses a vanilla index, the new shape is appended and resolvable.
    let trim_state = trim.default_state;
    let trim_shapes: Vec<BoundingBox> = trim_state.get_block_collision_shapes().collect();
    assert_eq!(trim_shapes.len(), 1);
    assert_eq!(
        (trim_shapes[0].min, trim_shapes[0].max),
        (slab_shape().min, slab_shape().max)
    );
    assert!(pumpkin_data::block_properties::has_random_ticks(
        trim_state.id
    ));
    assert!(!pumpkin_data::block_properties::has_random_ticks(
        drawer.default_state.id
    ));

    // Items.
    let item = Item::from_registry_key("testmod:drawer").expect("item registered");
    assert_eq!(item.id, vanilla_item_count);
    assert_eq!(Item::from_id(item.id).map(|i| i.id), Some(item.id));
    assert_eq!(
        Item::from_registry_key("minecraft:stick").map(|i| i.id),
        Some(Item::STICK.id)
    );

    // Tags: the modded block joined a vanilla tag, vanilla members kept their membership.
    assert!(drawer.has_tag(&tag::Block::MINECRAFT_MINEABLE_AXE));
    assert!(drawer.id.has_tag(tag::Block::MINECRAFT_MINEABLE_AXE));
    assert!(Block::CHEST.has_tag(&tag::Block::MINECRAFT_MINEABLE_AXE));
    assert!(!trim.has_tag(&tag::Block::MINECRAFT_MINEABLE_AXE));
    assert_eq!(drawer.is_tagged_with("minecraft:mineable/axe"), Some(true));
    let axe_ids = tag::get_tag_ids(RegistryKey::Block, "minecraft:mineable/axe").expect("tag");
    assert_eq!(
        axe_ids.len(),
        tag::Block::MINECRAFT_MINEABLE_AXE.1.len() + 1,
        "exactly one entry added"
    );
    // A new tag, including an expanded vanilla tag reference.
    assert_eq!(drawer.is_tagged_with("testmod:storage"), Some(true));
    assert_eq!(Block::CHEST.is_tagged_with("testmod:storage"), Some(true));
    assert_eq!(
        Block::OAK_LOG.is_tagged_with("#testmod:storage"),
        Some(true)
    );
    assert_eq!(Block::STONE.is_tagged_with("testmod:storage"), Some(false));

    // Only one freeze per process.
    assert_eq!(
        DynamicRegistriesBuilder::new().freeze(),
        Err(FreezeError::AlreadyFrozen)
    );
}
