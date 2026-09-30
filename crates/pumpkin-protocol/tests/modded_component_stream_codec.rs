//! Modded component types with their own stream codec (`networkSynchronized(...)`) are written in
//! the mod's format, from a plugin's description. Runs as its own binary because it freezes the
//! modded registries.

#![allow(clippy::expect_used, clippy::too_many_lines, reason = "test code")]

use std::borrow::Cow;

use pumpkin_data::dynamic::DynamicRegistriesBuilder;
use pumpkin_data::dynamic::names::SyncedRegistry;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::modded_component::{ComponentStreamCodec, StreamCodecNode};
use pumpkin_util::version::JavaMinecraftVersion;

#[test]
fn modded_components_use_their_stream_codec() {
    let mut builder = DynamicRegistriesBuilder::new();
    builder.add_entry(SyncedRegistry::DataComponentType, "testmod:binding");
    builder.add_entry(SyncedRegistry::DataComponentType, "testmod:frame");
    builder.freeze().expect("freeze");
    let binding_id = SyncedRegistry::DataComponentType.vanilla_len();
    let frame_id = binding_id + 1;

    // Storage Drawers' `ControllerBinding.STREAM_CODEC`: bool, then three ints.
    ComponentStreamCodec::register(
        binding_id,
        ComponentStreamCodec {
            nodes: vec![
                StreamCodecNode::Composite(vec![
                    ("valid".into(), 1),
                    ("x".into(), 2),
                    ("y".into(), 2),
                    ("z".into(), 2),
                ]),
                StreamCodecNode::Bool,
                StreamCodecNode::Int,
            ],
        },
    )
    .expect("register binding");
    // `FrameData.STREAM_CODEC`: four `ItemStack.OPTIONAL_STREAM_CODEC`.
    ComponentStreamCodec::register(
        frame_id,
        ComponentStreamCodec {
            nodes: vec![
                StreamCodecNode::Composite(vec![
                    ("base".into(), 1),
                    ("side".into(), 1),
                    ("trim".into(), 1),
                    ("front".into(), 1),
                ]),
                StreamCodecNode::OptionalItemStack,
            ],
        },
    )
    .expect("register frame");
    assert!(
        ComponentStreamCodec::register(
            frame_id,
            ComponentStreamCodec {
                nodes: vec![StreamCodecNode::List(3)],
            },
        )
        .is_err(),
        "a child index past the end is refused"
    );

    let mut binding = NbtCompound::new();
    binding.put_bool("valid", true);
    binding.put_int("x", 1);
    binding.put_int("y", -2);
    binding.put_int("z", 300);
    let mut stack = ItemStack::new(1, &Item::STONE);
    stack
        .unknown_patch
        .push((binding_id, Some(NbtTag::Compound(binding))));

    let mut bytes = Vec::new();
    ItemStackSerializer(Cow::Borrowed(&stack))
        .write(&mut bytes)
        .expect("write");
    let mut expected = vec![1];
    expected.push(u8::try_from(Item::STONE.id).expect("stone id fits a one-byte varint"));
    expected.extend([1, 0]);
    expected.extend(varint(i32::from(binding_id)));
    expected.push(1);
    expected.extend(1i32.to_be_bytes());
    expected.extend((-2i32).to_be_bytes());
    expected.extend(300i32.to_be_bytes());
    assert_eq!(bytes, expected);
    let read = ItemStackSerializer::read(&mut bytes.as_slice())
        .expect("read")
        .to_stack();
    assert!(read.are_equal(&stack));

    // A frame with a base and an empty side, trim and front.
    let mut base = NbtCompound::new();
    ItemStack::new(1, &Item::OAK_PLANKS).write_item_stack(&mut base);
    let mut frame = NbtCompound::new();
    frame.put_compound("base", base);
    frame.put_compound("side", NbtCompound::new());
    frame.put_compound("trim", NbtCompound::new());
    frame.put_compound("front", NbtCompound::new());
    let mut stack = ItemStack::new(1, &Item::STONE);
    stack
        .unknown_patch
        .push((frame_id, Some(NbtTag::Compound(frame))));

    let mut bytes = Vec::new();
    ItemStackSerializer(Cow::Borrowed(&stack))
        .write(&mut bytes)
        .expect("write");
    let mut expected = vec![1];
    expected.push(u8::try_from(Item::STONE.id).expect("stone id fits a one-byte varint"));
    expected.extend([1, 0]);
    expected.extend(varint(i32::from(frame_id)));
    expected.push(1);
    expected.extend(varint(i32::from(Item::OAK_PLANKS.id)));
    expected.extend([0, 0, 0, 0, 0]);
    assert_eq!(bytes, expected);
    let read = ItemStackSerializer::read(&mut bytes.as_slice())
        .expect("read")
        .to_stack();
    assert!(read.are_equal(&stack));

    // The length-prefixed form (`ItemStack.OPTIONAL_UNTRUSTED_STREAM_CODEC`, creative slots).
    let mut bytes = Vec::new();
    ItemStackSerializer(Cow::Borrowed(&stack))
        .write_length_prefixed_with_version(&mut bytes, &JavaMinecraftVersion::V_26_3)
        .expect("write");
    let read = ItemStackSerializer::read_length_prefixed_optional(&mut bytes.as_slice())
        .expect("read")
        .to_stack();
    assert!(read.are_equal(&stack));
}

fn varint(mut value: i32) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let byte = (value & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value == 0 {
            out.push(byte);
            return out;
        }
        out.push(byte | 0x80);
    }
}
