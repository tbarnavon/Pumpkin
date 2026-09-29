//! Anvil save/load of chunks with modded blocks, and with blocks whose mod is missing.
//!
//! The registry overlay is process-global and frozen once, so both cases share one test.
#![allow(clippy::expect_used, clippy::too_many_lines)]

use bytes::Bytes;
use pumpkin_data::block_properties::NoteblockInstrument;
use pumpkin_data::block_state::PistonBehavior;
use pumpkin_data::dynamic::DynamicRegistriesBuilder;
use pumpkin_data::dynamic::blocks::{BlockDef, StateDef};
use pumpkin_data::{Block, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::format::anvil::SingleChunkDataSerializer;
use pumpkin_world::chunk::format::unknown_blocks::STAND_IN;

fn drawer_def() -> BlockDef {
    let states = ["north", "south", "west", "east"]
        .iter()
        .map(|facing| StateDef {
            values: vec![(*facing).to_string()],
            state_flags: 0,
            side_flags: 0,
            instrument: NoteblockInstrument::Harp,
            luminance: 0,
            piston_behavior: PistonBehavior::Block,
            hardness: 3.0,
            opacity: 15,
            collision_shapes: Vec::new(),
            outline_shapes: Vec::new(),
            block_entity_type: None,
            random_ticks: false,
        })
        .collect();
    BlockDef {
        name: "testmod:drawer".to_string(),
        hardness: 3.0,
        blast_resistance: 3.0,
        map_color: 0,
        slipperiness: 0.6,
        velocity_multiplier: 1.0,
        jump_velocity_multiplier: 1.0,
        item_id: 0,
        flammable: None,
        experience: None,
        properties: vec![(
            "facing".to_string(),
            vec!["north".into(), "south".into(), "west".into(), "east".into()],
        )],
        states,
        default_state: 0,
    }
}

fn palette_entry(name: &str, props: &[(&str, &str)]) -> NbtTag {
    let mut entry = NbtCompound::new();
    entry.put_string("Name", name.to_string());
    if !props.is_empty() {
        let mut properties = NbtCompound::new();
        for (key, value) in props {
            properties.put_string(key, (*value).to_string());
        }
        entry.put_compound("Properties", properties);
    }
    NbtTag::Compound(entry)
}

/// One section at Y=0 whose palette is `palette`, with palette index 1 at (0,0,0) and 0 elsewhere.
fn chunk_bytes(palette: Vec<NbtTag>) -> Bytes {
    let mut block_states = NbtCompound::new();
    block_states.put("palette", NbtTag::List(palette));
    let mut data = vec![0i64; 256];
    data[0] = 1;
    block_states.put("data", NbtTag::LongArray(data));

    let mut biomes = NbtCompound::new();
    biomes.put(
        "palette",
        NbtTag::List(vec![NbtTag::String("minecraft:plains".into())]),
    );

    let mut section = NbtCompound::new();
    section.put_int("Y", 0);
    section.put("block_states", NbtTag::Compound(block_states));
    section.put("biomes", NbtTag::Compound(biomes));

    let mut root = NbtCompound::new();
    root.put_int("DataVersion", 4903);
    root.put_int("xPos", 0);
    root.put_int("zPos", 0);
    root.put_string("Status", "minecraft:full".to_string());
    root.put("sections", NbtTag::List(vec![NbtTag::Compound(section)]));
    pumpkin_nbt::Nbt::new(String::new(), root).write()
}

fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
}

#[test]
fn modded_and_unknown_blocks_survive_save_and_load() {
    let mut builder = DynamicRegistriesBuilder::new();
    builder.add_block(drawer_def());
    builder.freeze().expect("freeze");
    let drawer = Block::from_name("testmod:drawer").expect("registered");
    let east = drawer
        .state_from_properties(&[("facing", "east")])
        .expect("state")
        .id;

    // A registered modded block round-trips by name and properties.
    let air = palette_entry("minecraft:air", &[]);
    let bytes = chunk_bytes(vec![
        air.clone(),
        palette_entry("testmod:drawer", &[("facing", "east")]),
    ]);
    let chunk = ChunkData::from_bytes(&bytes, Vector2::new(0, 0)).expect("chunk parses");
    assert_eq!(chunk.section.get_block_absolute_y(0, 0, 0), Some(east));
    assert_eq!(
        chunk.section.get_block_absolute_y(1, 0, 0),
        Some(BlockStateId::AIR)
    );
    chunk
        .section
        .set_block_absolute_y(2, 0, 0, drawer.default_state.id);

    let saved = chunk.to_bytes().expect("chunk saves");
    assert!(contains(&saved, "testmod:drawer"));
    assert!(!contains(&saved, "minecraft:testmod"));
    let reloaded = ChunkData::from_bytes(&saved, Vector2::new(0, 0)).expect("saved chunk parses");
    assert_eq!(reloaded.section.get_block_absolute_y(0, 0, 0), Some(east));
    assert_eq!(
        reloaded.section.get_block_absolute_y(2, 0, 0),
        Some(drawer.default_state.id)
    );
    assert_eq!(
        reloaded.section.get_block_absolute_y(3, 0, 0),
        Some(BlockStateId::AIR)
    );

    // A block from a missing mod loads as the stand-in and is written back unchanged.
    let bytes = chunk_bytes(vec![
        air,
        palette_entry("missingmod:widget", &[("color", "blue"), ("lit", "true")]),
    ]);
    let chunk = ChunkData::from_bytes(&bytes, Vector2::new(0, 0)).expect("chunk parses");
    assert_eq!(chunk.section.get_block_absolute_y(0, 0, 0), Some(STAND_IN));
    assert_eq!(
        chunk.section.get_block_absolute_y(1, 0, 0),
        Some(BlockStateId::AIR)
    );

    let saved = chunk.to_bytes().expect("chunk saves");
    assert!(contains(&saved, "missingmod:widget"));
    let reloaded = ChunkData::from_bytes(&saved, Vector2::new(0, 0)).expect("saved chunk parses");
    assert_eq!(
        reloaded.section.get_block_absolute_y(0, 0, 0),
        Some(STAND_IN)
    );
    let resaved = reloaded.to_bytes().expect("chunk saves again");
    assert!(contains(&resaved, "missingmod:widget"));
    assert!(contains(&resaved, "blue"));

    // Once the stand-in is replaced, the original entry is dropped instead of resurrected.
    reloaded
        .section
        .set_block_absolute_y(0, 0, 0, Block::STONE.default_state.id);
    let replaced = reloaded.to_bytes().expect("chunk saves");
    assert!(!contains(&replaced, "missingmod:widget"));
}
