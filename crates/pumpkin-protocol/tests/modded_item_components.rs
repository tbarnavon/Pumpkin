//! Item stacks keep component types Pumpkin has no implementation of (modded ones), on the
//! network and on disk. Runs as its own binary because it freezes the modded registries.

#![allow(clippy::expect_used, reason = "test code")]

use std::borrow::Cow;

use pumpkin_data::dynamic::DynamicRegistriesBuilder;
use pumpkin_data::dynamic::names::SyncedRegistry;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;

#[test]
fn modded_components_survive_network_and_disk() {
    let mut builder = DynamicRegistriesBuilder::new();
    builder.add_entry(SyncedRegistry::DataComponentType, "testmod:count");
    builder.freeze().expect("freeze");
    let id = SyncedRegistry::DataComponentType.vanilla_len();

    let mut value = NbtCompound::new();
    value.put_int("__count", 300);
    let mut stack = ItemStack::new(1, &Item::STONE);
    stack
        .unknown_patch
        .push((id, Some(NbtTag::Compound(value.clone()))));

    let mut bytes = Vec::new();
    ItemStackSerializer(Cow::Borrowed(&stack))
        .write(&mut bytes)
        .expect("write");

    // count, item id, 1 added, 0 removed, then the component id and the value as a network NBT
    // tag, as vanilla's `ByteBufCodecs.fromCodecWithRegistries` writes it.
    let mut expected = vec![1];
    let item_id = u8::try_from(Item::STONE.id).expect("stone id fits a one-byte varint");
    expected.push(item_id);
    expected.extend([1, 0]);
    expected.extend(varint(i32::from(id)));
    expected.extend([10, 3, 0, 7]);
    expected.extend(b"__count");
    expected.extend(300i32.to_be_bytes());
    expected.push(0);
    assert_eq!(bytes, expected);

    let read = ItemStackSerializer::read(&mut bytes.as_slice())
        .expect("read")
        .to_stack();
    assert!(read.are_equal(&stack));

    let mut saved = NbtCompound::new();
    stack.write_item_stack(&mut saved);
    let components = saved.get_compound("components").expect("components");
    assert_eq!(
        components.get_compound("testmod:count"),
        Some(&value),
        "saved under the component's name"
    );
    let loaded = ItemStack::read_item_stack(&saved).expect("load");
    assert!(loaded.are_equal(&stack));
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
