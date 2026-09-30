//! Stream codecs of modded data components.
//!
//! Pumpkin keeps a modded component's value as the NBT its persistent codec writes. On the network,
//! a component type without `networkSynchronized(...)` uses vanilla's default stream codec (the
//! value as a network NBT tag), but a mod can give its type its own `StreamCodec`, usually a
//! `StreamCodec.composite` of primitive codecs. A plugin describes such a codec once, as a tree of
//! [`StreamCodecNode`]s, and the item stack serializer translates between the stored NBT and the
//! mod's bytes with it.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::codec::var_long::VarLong;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};

/// One node of a stream codec description. Nodes refer to their children by index in
/// [`ComponentStreamCodec::nodes`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamCodecNode {
    /// `ByteBufCodecs.BOOL`; NBT byte 0 or 1.
    Bool,
    /// `ByteBufCodecs.BYTE`.
    Byte,
    /// `ByteBufCodecs.SHORT`.
    Short,
    /// `ByteBufCodecs.INT`.
    Int,
    /// `ByteBufCodecs.LONG`.
    Long,
    /// `ByteBufCodecs.FLOAT`.
    Float,
    /// `ByteBufCodecs.DOUBLE`.
    Double,
    /// `ByteBufCodecs.VAR_INT`; NBT int.
    VarInt,
    /// `ByteBufCodecs.VAR_LONG`; NBT long.
    VarLong,
    /// `ByteBufCodecs.STRING_UTF8`.
    String,
    /// `ByteBufCodecs.TAG`: any NBT value as a network NBT tag.
    Nbt,
    /// `ItemStack.STREAM_CODEC`; NBT as `ItemStack.CODEC`.
    ItemStack,
    /// `ItemStack.OPTIONAL_STREAM_CODEC`; NBT as `ItemStack.OPTIONAL_CODEC` (`{}` when empty).
    OptionalItemStack,
    /// `UUIDUtil.STREAM_CODEC` (two longs); NBT int array of 4.
    Uuid,
    /// `BlockPos.STREAM_CODEC` (packed long); NBT int array of 3.
    BlockPos,
    /// `StreamCodec.composite`: an NBT compound whose fields are written in this order, each
    /// by its key and node.
    Composite(Vec<(String, u32)>),
    /// `ByteBufCodecs.list()`: an NBT list, written as a var-int length and the elements.
    List(u32),
    /// `ByteBufCodecs.optional()`: a bool, then the value when present. In a composite, an absent
    /// field is empty.
    Optional(u32),
}

/// A modded component type's stream codec. The root is node 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentStreamCodec {
    pub nodes: Vec<StreamCodecNode>,
}

static CODECS: LazyLock<RwLock<HashMap<u16, Arc<ComponentStreamCodec>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

impl ComponentStreamCodec {
    /// Checks that every child index points at an existing node and that the tree has no cycle.
    pub fn validate(&self) -> Result<(), String> {
        fn visit(codec: &ComponentStreamCodec, index: u32, depth: usize) -> Result<(), String> {
            if depth > codec.nodes.len() {
                return Err("the stream codec description has a cycle".into());
            }
            let node = codec
                .nodes
                .get(index as usize)
                .ok_or_else(|| format!("stream codec node {index} doesn't exist"))?;
            match node {
                StreamCodecNode::Composite(fields) => {
                    for (_, child) in fields {
                        visit(codec, *child, depth + 1)?;
                    }
                    Ok(())
                }
                StreamCodecNode::List(child) | StreamCodecNode::Optional(child) => {
                    visit(codec, *child, depth + 1)
                }
                _ => Ok(()),
            }
        }
        visit(self, 0, 0)
    }

    /// Sets the stream codec of a modded component type, by raw id. Replaces an earlier one.
    pub fn register(raw_id: u16, codec: Self) -> Result<(), String> {
        codec.validate()?;
        CODECS
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(raw_id, Arc::new(codec));
        Ok(())
    }

    /// The stream codec of a modded component type, if a plugin gave it one.
    #[must_use]
    pub fn get(raw_id: u16) -> Option<Arc<Self>> {
        let codecs = CODECS
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if codecs.is_empty() {
            return None;
        }
        codecs.get(&raw_id).cloned()
    }

    fn node(&self, index: u32) -> &StreamCodecNode {
        &self.nodes[index as usize]
    }

    /// Writes a stored value in the mod's stream format.
    pub fn encode(
        &self,
        value: &NbtTag,
        write: &mut impl NetworkWriteExt,
    ) -> Result<(), WritingError> {
        self.encode_node(0, value, write)
    }

    /// Reads a value in the mod's stream format, as the NBT Pumpkin stores.
    pub fn decode(&self, read: &mut impl NetworkReadExt) -> Result<NbtTag, ReadingError> {
        self.decode_node(0, read)
    }

    fn encode_node(
        &self,
        index: u32,
        value: &NbtTag,
        write: &mut impl NetworkWriteExt,
    ) -> Result<(), WritingError> {
        let mismatch = |expected: &str| {
            WritingError::Message(format!(
                "modded component value {value:?} doesn't match the stream codec ({expected})"
            ))
        };
        match self.node(index) {
            StreamCodecNode::Bool => {
                write.write_bool(as_i64(value).ok_or_else(|| mismatch("bool"))? != 0)
            }
            StreamCodecNode::Byte => {
                write.write_i8(as_i64(value).ok_or_else(|| mismatch("byte"))? as i8)
            }
            StreamCodecNode::Short => {
                write.write_i16_be(as_i64(value).ok_or_else(|| mismatch("short"))? as i16)
            }
            StreamCodecNode::Int => {
                write.write_i32_be(as_i64(value).ok_or_else(|| mismatch("int"))? as i32)
            }
            StreamCodecNode::Long => {
                write.write_i64_be(as_i64(value).ok_or_else(|| mismatch("long"))?)
            }
            StreamCodecNode::Float => {
                write.write_f32_be(as_f64(value).ok_or_else(|| mismatch("float"))? as f32)
            }
            StreamCodecNode::Double => {
                write.write_f64_be(as_f64(value).ok_or_else(|| mismatch("double"))?)
            }
            StreamCodecNode::VarInt => write.write_var_int(&VarInt(
                as_i64(value).ok_or_else(|| mismatch("var-int"))? as i32,
            )),
            StreamCodecNode::VarLong => {
                write.write_var_long(&VarLong(as_i64(value).ok_or_else(|| mismatch("var-long"))?))
            }
            StreamCodecNode::String => match value {
                NbtTag::String(s) => write.write_string(s),
                _ => Err(mismatch("string")),
            },
            StreamCodecNode::Nbt => write.write_nbt(value.clone()),
            StreamCodecNode::ItemStack | StreamCodecNode::OptionalItemStack => {
                let stack = match value {
                    NbtTag::Compound(compound) if !compound.is_empty() => {
                        ItemStack::read_item_stack(compound)
                            .ok_or_else(|| mismatch("item stack"))?
                    }
                    NbtTag::Compound(_) => ItemStack::EMPTY.clone(),
                    _ => return Err(mismatch("item stack")),
                };
                if stack.is_empty() && *self.node(index) == StreamCodecNode::ItemStack {
                    return Err(mismatch("non-empty item stack"));
                }
                ItemStackSerializer(Cow::Owned(stack)).write(write)
            }
            StreamCodecNode::Uuid => match value {
                NbtTag::IntArray(ints) if ints.len() == 4 => {
                    let value = ints
                        .iter()
                        .fold(0u128, |acc, i| (acc << 32) | u128::from(*i as u32));
                    write.write_u64_be((value >> 64) as u64)?;
                    write.write_u64_be(value as u64)
                }
                _ => Err(mismatch("uuid")),
            },
            StreamCodecNode::BlockPos => match value {
                NbtTag::IntArray(ints) if ints.len() == 3 => {
                    write.write_i64_be(BlockPos::new(ints[0], ints[1], ints[2]).as_long())
                }
                _ => Err(mismatch("block pos")),
            },
            StreamCodecNode::Composite(fields) => {
                let NbtTag::Compound(compound) = value else {
                    return Err(mismatch("compound"));
                };
                self.encode_composite(fields, compound, write)
            }
            StreamCodecNode::List(child) => {
                let elements: Vec<NbtTag> = match value {
                    NbtTag::List(list) => list.clone(),
                    NbtTag::ByteArray(a) => a.iter().map(|b| NbtTag::Byte(*b)).collect(),
                    NbtTag::IntArray(a) => a.iter().map(|i| NbtTag::Int(*i)).collect(),
                    NbtTag::LongArray(a) => a.iter().map(|l| NbtTag::Long(*l)).collect(),
                    _ => return Err(mismatch("list")),
                };
                write.write_var_int(&VarInt(elements.len() as i32))?;
                for element in &elements {
                    self.encode_node(*child, element, write)?;
                }
                Ok(())
            }
            StreamCodecNode::Optional(child) => {
                write.write_bool(true)?;
                self.encode_node(*child, value, write)
            }
        }
    }

    fn encode_composite(
        &self,
        fields: &[(String, u32)],
        compound: &NbtCompound,
        write: &mut impl NetworkWriteExt,
    ) -> Result<(), WritingError> {
        for (key, child) in fields {
            match compound.child_tags.get(key.as_str()) {
                Some(field) => self.encode_node(*child, field, write)?,
                None if matches!(self.node(*child), StreamCodecNode::Optional(_)) => {
                    write.write_bool(false)?;
                }
                None => {
                    return Err(WritingError::Message(format!(
                        "modded component value has no field `{key}`"
                    )));
                }
            }
        }
        Ok(())
    }

    fn decode_node(
        &self,
        index: u32,
        read: &mut impl NetworkReadExt,
    ) -> Result<NbtTag, ReadingError> {
        Ok(match self.node(index) {
            StreamCodecNode::Bool => NbtTag::Byte(i8::from(read.get_bool()?)),
            StreamCodecNode::Byte => NbtTag::Byte(read.get_i8()?),
            StreamCodecNode::Short => NbtTag::Short(read.get_i16_be()?),
            StreamCodecNode::Int => NbtTag::Int(read.get_i32_be()?),
            StreamCodecNode::Long => NbtTag::Long(read.get_i64_be()?),
            StreamCodecNode::Float => NbtTag::Float(read.get_f32_be()?),
            StreamCodecNode::Double => NbtTag::Double(read.get_f64_be()?),
            StreamCodecNode::VarInt => NbtTag::Int(read.get_var_int()?.0),
            StreamCodecNode::VarLong => NbtTag::Long(read.get_var_long()?.0),
            StreamCodecNode::String => NbtTag::String(read.get_str()?),
            StreamCodecNode::Nbt => read
                .get_nbt_with_version(&pumpkin_util::version::JavaMinecraftVersion::V_26_3)?
                .unwrap_or(NbtTag::End),
            StreamCodecNode::ItemStack | StreamCodecNode::OptionalItemStack => {
                let stack = ItemStackSerializer::read(read)?.to_stack();
                let mut compound = NbtCompound::new();
                if !stack.is_empty() {
                    stack.write_item_stack(&mut compound);
                } else if *self.node(index) == StreamCodecNode::ItemStack {
                    return Err(ReadingError::Message("empty item stack".into()));
                }
                NbtTag::Compound(compound)
            }
            StreamCodecNode::Uuid => {
                let value = (u128::from(read.get_u64_be()?) << 64) | u128::from(read.get_u64_be()?);
                NbtTag::IntArray(
                    (0..4)
                        .map(|i| (value >> (96 - 32 * i)) as u32 as i32)
                        .collect(),
                )
            }
            StreamCodecNode::BlockPos => {
                let pos = BlockPos::from_i64(read.get_i64_be()?);
                NbtTag::IntArray(vec![pos.0.x, pos.0.y, pos.0.z])
            }
            StreamCodecNode::Composite(fields) => {
                let mut compound = NbtCompound::new();
                for (key, child) in fields {
                    if let StreamCodecNode::Optional(inner) = self.node(*child) {
                        if read.get_bool()? {
                            let value = self.decode_node(*inner, read)?;
                            compound.put(key, value);
                        }
                    } else {
                        let value = self.decode_node(*child, read)?;
                        compound.put(key, value);
                    }
                }
                NbtTag::Compound(compound)
            }
            StreamCodecNode::List(child) => {
                let len = read.get_var_int()?.0;
                let len = usize::try_from(len)
                    .map_err(|_| ReadingError::Message("negative list length".into()))?;
                if len > 65536 {
                    return Err(ReadingError::TooLarge("modded component list".into()));
                }
                let mut list = Vec::with_capacity(len.min(256));
                for _ in 0..len {
                    list.push(self.decode_node(*child, read)?);
                }
                NbtTag::List(list)
            }
            StreamCodecNode::Optional(child) => {
                if read.get_bool()? {
                    self.decode_node(*child, read)?
                } else {
                    NbtTag::End
                }
            }
        })
    }
}

fn as_i64(tag: &NbtTag) -> Option<i64> {
    Some(match tag {
        NbtTag::Byte(v) => i64::from(*v),
        NbtTag::Short(v) => i64::from(*v),
        NbtTag::Int(v) => i64::from(*v),
        NbtTag::Long(v) => *v,
        _ => return None,
    })
}

fn as_f64(tag: &NbtTag) -> Option<f64> {
    Some(match tag {
        NbtTag::Float(v) => f64::from(*v),
        NbtTag::Double(v) => *v,
        other => as_i64(other)? as f64,
    })
}
