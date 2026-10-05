use crate::{
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    errors::error_types::CommandErrorType,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};
use pumpkin_data::{Block, BlockStateId, translation};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::text::TextComponent;

use crate::argument_types::block_predicate::{parse_nbt, parse_properties};

pub const INVALID_BLOCK_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_ID_INVALID,
    translation::java::ARGUMENT_BLOCK_ID_INVALID,
);

pub const UNKNOWN_PROPERTY_ERROR_TYPE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_UNKNOWN,
    translation::java::ARGUMENT_BLOCK_PROPERTY_UNKNOWN,
);

pub const INVALID_PROPERTY_ERROR_TYPE: CommandErrorType<3> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_INVALID,
    translation::java::ARGUMENT_BLOCK_PROPERTY_INVALID,
);

/// A parsed block state (`BlockStateArgument`): `id[property=value,...]{block entity NBT}`.
#[derive(Debug, Clone)]
pub struct BlockStateInput {
    pub block: &'static Block,
    pub state: BlockStateId,
    pub nbt: Option<NbtCompound>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlockArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for BlockArgumentType {
    type Item = BlockStateInput;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        while let Some(c) = reader.peek() {
            if c.is_alphanumeric() || c == '_' || c == ':' || c == '/' || c == '.' || c == '-' {
                reader.skip();
            } else {
                break;
            }
        }
        let block_name = &reader.string()[start..reader.cursor()];
        let normalized = if block_name.contains(':') {
            block_name.to_string()
        } else {
            format!("minecraft:{block_name}")
        };

        let block = Block::from_name(&normalized).ok_or_else(|| {
            INVALID_BLOCK_ERROR_TYPE.create(reader, TextComponent::text(normalized.clone()))
        })?;
        let properties = parse_properties(reader)?;
        let state = if properties.is_empty() {
            block.default_state.id
        } else {
            // `BlockStateParser`: the given properties set on the default state.
            let defaults = block
                .properties(block.default_state.id)
                .map(|properties| properties.to_props())
                .unwrap_or_default();
            for name in properties.keys() {
                if !defaults.iter().any(|(default, _)| default == name) {
                    return Err(UNKNOWN_PROPERTY_ERROR_TYPE.create(
                        reader,
                        TextComponent::text(normalized),
                        TextComponent::text(name.clone()),
                    ));
                }
            }
            let wanted: Vec<(&str, &str)> = defaults
                .iter()
                .map(|(name, default)| {
                    (
                        *name,
                        properties.get(*name).map_or(*default, String::as_str),
                    )
                })
                .collect();
            let Some(state) = block.state_from_properties(&wanted) else {
                // Name the first property whose value no state has.
                let (name, value) = properties
                    .iter()
                    .find(|(name, value)| {
                        !block.states.iter().any(|state| {
                            block.properties(state.id).is_some_and(|properties| {
                                properties
                                    .to_props()
                                    .iter()
                                    .any(|(n, v)| n == name && v == value)
                            })
                        })
                    })
                    .unwrap_or_else(|| properties.iter().next().expect("not empty"));
                return Err(INVALID_PROPERTY_ERROR_TYPE.create(
                    reader,
                    TextComponent::text(normalized),
                    TextComponent::text(value.clone()),
                    TextComponent::text(name.clone()),
                ));
            };
            state.id
        };
        let nbt = parse_nbt(reader)?;
        Ok(BlockStateInput { block, state, nbt })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::BlockState
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.build()
    }
}

impl BlockArgumentType {
    pub fn get<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<&'static Block, CommandSyntaxError> {
        Ok(Self::get_state(context, name)?.block)
    }

    /// The block with the state and block entity NBT the argument named.
    pub fn get_state<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<BlockStateInput, CommandSyntaxError> {
        context.get_argument::<BlockStateInput>(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::DummySource;

    fn parse(input: &str) -> Result<BlockStateInput, CommandSyntaxError> {
        ArgumentType::<DummySource>::parse(&BlockArgumentType, &mut StringReader::new(input))
    }

    #[test]
    fn parses_properties_and_nbt() {
        let leaves = parse("oak_leaves[persistent=true]").expect("leaves");
        assert_eq!(leaves.block.id, Block::OAK_LEAVES.id);
        let expected = Block::OAK_LEAVES
            .states
            .iter()
            .find(|state| {
                Block::OAK_LEAVES
                    .properties(state.id)
                    .is_some_and(|p| p.to_props().contains(&("persistent", "true")))
                    && Block::OAK_LEAVES.properties(state.id).is_some_and(|p| {
                        let default = Block::OAK_LEAVES
                            .properties(Block::OAK_LEAVES.default_state.id)
                            .expect("props")
                            .to_props();
                        p.to_props()
                            .iter()
                            .all(|kv| kv.0 == "persistent" || default.contains(kv))
                    })
            })
            .expect("state");
        assert_eq!(leaves.state, expected.id);
        assert_ne!(leaves.state, Block::OAK_LEAVES.default_state.id);

        let stone = parse("minecraft:stone").expect("stone");
        assert_eq!(stone.state, Block::STONE.default_state.id);
        assert!(stone.nbt.is_none());

        let chest = parse("chest{Lock:\"k\"}").expect("chest");
        assert!(chest.nbt.is_some());

        assert!(parse("oak_leaves[nope=true]").is_err());
        assert!(parse("oak_leaves[persistent=maybe]").is_err());
    }
}
