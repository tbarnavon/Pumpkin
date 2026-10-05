//! Adventure mode's block rules.
//!
//! A player who may not build only breaks blocks its held item's `can_break` matches, and only
//! uses items on blocks their `can_place_on` matches (`Player.blockActionRestricted`,
//! `ItemStack.useOn`).

use pumpkin_command::snbt::SnbtParser;
use pumpkin_command::string_reader::StringReader;
use pumpkin_data::data_component_impl::{CanBreakImpl, CanPlaceOnImpl};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_data::{Block, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;

use super::Player;
use crate::world::World;

impl Player {
    /// `Player.blockActionRestricted`: whether this player may not break the block at `pos`.
    #[must_use]
    pub fn block_action_restricted(&self, world: &World, pos: &BlockPos) -> bool {
        match self.gamemode.load() {
            GameMode::Spectator => true,
            GameMode::Adventure if !self.may_build() => {
                let held = self.inventory().held_item();
                held.is_empty() || !can_break(&held, world, pos)
            }
            _ => false,
        }
    }
}

/// `ItemStack.canBreakBlockInAdventureMode`.
#[must_use]
pub fn can_break(stack: &ItemStack, world: &World, pos: &BlockPos) -> bool {
    stack
        .get_data_component::<CanBreakImpl>()
        .is_some_and(|component| matches_any(&component.predicate, world, pos))
}

/// `ItemStack.canPlaceOnBlockInAdventureMode`.
#[must_use]
pub fn can_place_on(stack: &ItemStack, world: &World, pos: &BlockPos) -> bool {
    stack
        .get_data_component::<CanPlaceOnImpl>()
        .is_some_and(|component| matches_any(&component.predicate, world, pos))
}

/// An `AdventureModePredicate` as saved: a list of block predicates, one predicate, or
/// 1.21.1's `{predicates, show_in_tooltip}`. Matches when any predicate does.
fn matches_any(predicate: &NbtTag, world: &World, pos: &BlockPos) -> bool {
    let predicates: &[NbtTag] = match predicate {
        NbtTag::Compound(full) if full.get("predicates").is_some() => {
            full.get_list("predicates").unwrap_or_default()
        }
        NbtTag::Compound(_) => std::slice::from_ref(predicate),
        NbtTag::List(list) => list,
        _ => return false,
    };
    predicates.iter().any(|predicate| {
        predicate
            .extract_compound()
            .is_some_and(|predicate| block_predicate_matches(predicate, world, pos))
    })
}

/// `BlockPredicate.matches`: the block (an id, a `#tag` or a list of ids), its state
/// properties, and its block entity's NBT.
fn block_predicate_matches(predicate: &NbtCompound, world: &World, pos: &BlockPos) -> bool {
    let (block, state) = world.get_block_and_state(pos);
    if let Some(blocks) = predicate.get("blocks")
        && !blocks_match(blocks, block)
    {
        return false;
    }
    if let Some(properties) = predicate.get_compound("state")
        && !state_matches(properties, block, state.id)
    {
        return false;
    }
    if let Some(nbt) = predicate.get("nbt") {
        let expected = match nbt {
            NbtTag::Compound(compound) => Some(compound.clone()),
            NbtTag::String(snbt) => {
                match SnbtParser::parse_for_commands(&mut StringReader::new(snbt.to_string())) {
                    Ok(NbtTag::Compound(compound)) => Some(compound),
                    _ => None,
                }
            }
            _ => None,
        };
        let Some(expected) = expected else {
            return false;
        };
        let Some(block_entity) = world.get_block_entity(pos) else {
            return false;
        };
        let mut actual = NbtCompound::new();
        block_entity.write_nbt(&mut actual);
        if !nbt_contains(&NbtTag::Compound(actual), &NbtTag::Compound(expected)) {
            return false;
        }
    }
    true
}

fn blocks_match(blocks: &NbtTag, block: &Block) -> bool {
    let matches_name = |name: &str| {
        name.strip_prefix('#').map_or_else(
            || Block::from_name(name).is_some_and(|named| named.id == block.id),
            |tag| {
                let tag = if tag.contains(':') {
                    tag.to_string()
                } else {
                    format!("minecraft:{tag}")
                };
                get_tag_ids(RegistryKey::Block, &tag)
                    .is_some_and(|ids| ids.contains(&block.id.as_u16()))
            },
        )
    };
    match blocks {
        NbtTag::String(name) => matches_name(name),
        NbtTag::List(names) => names
            .iter()
            .filter_map(NbtTag::extract_string)
            .any(matches_name),
        _ => false,
    }
}

/// `StatePropertiesPredicate`: each property equals a value, or lies in a `{min, max}` range.
fn state_matches(properties: &NbtCompound, block: &Block, state: BlockStateId) -> bool {
    let actual = block
        .properties(state)
        .map(|properties| properties.to_props())
        .unwrap_or_default();
    properties.child_tags.iter().all(|(name, matcher)| {
        let Some((_, value)) = actual
            .iter()
            .find(|(property, _)| *property == name.as_ref())
        else {
            return false;
        };
        match matcher {
            NbtTag::Compound(range) => {
                let bound = |key: &str| range.get(key).and_then(value_string);
                let in_range = |bound: Option<String>, below: bool| {
                    bound.is_none_or(|bound| match (value.parse::<i64>(), bound.parse::<i64>()) {
                        (Ok(value), Ok(bound)) => {
                            if below {
                                value >= bound
                            } else {
                                value <= bound
                            }
                        }
                        _ => *value == bound,
                    })
                };
                in_range(bound("min"), true) && in_range(bound("max"), false)
            }
            exact => value_string(exact).is_some_and(|expected| *value == expected),
        }
    })
}

fn value_string(value: &NbtTag) -> Option<String> {
    Some(match value {
        NbtTag::String(value) => value.to_string(),
        NbtTag::Byte(value) => match value {
            0 => "false".to_string(),
            1 => "true".to_string(),
            other => other.to_string(),
        },
        NbtTag::Short(value) => value.to_string(),
        NbtTag::Int(value) => value.to_string(),
        NbtTag::Long(value) => value.to_string(),
        _ => return None,
    })
}

/// `NbtUtils.compareNbt` with partial lists: every key of `expected` is in `actual` with a
/// matching value, and every element of an expected list matches one of the actual list.
fn nbt_contains(actual: &NbtTag, expected: &NbtTag) -> bool {
    match (actual, expected) {
        (NbtTag::Compound(actual), NbtTag::Compound(expected)) => {
            expected.child_tags.iter().all(|(key, value)| {
                actual
                    .get(key)
                    .is_some_and(|actual| nbt_contains(actual, value))
            })
        }
        (NbtTag::List(actual), NbtTag::List(expected)) => {
            if expected.is_empty() {
                return actual.is_empty();
            }
            expected
                .iter()
                .all(|expected| actual.iter().any(|actual| nbt_contains(actual, expected)))
        }
        (actual, expected) => actual == expected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_match_ids_tags_and_lists() {
        let stone = &Block::STONE;
        assert!(blocks_match(
            &NbtTag::String("minecraft:stone".into()),
            stone
        ));
        assert!(blocks_match(&NbtTag::String("stone".into()), stone));
        assert!(!blocks_match(
            &NbtTag::String("minecraft:dirt".into()),
            stone
        ));
        assert!(blocks_match(
            &NbtTag::String("#minecraft:base_stone_overworld".into()),
            stone
        ));
        assert!(blocks_match(
            &NbtTag::List(vec![
                NbtTag::String("minecraft:dirt".into()),
                NbtTag::String("minecraft:stone".into()),
            ]),
            stone
        ));
    }

    #[test]
    fn state_matches_exact_values_and_ranges() {
        let wheat = &Block::WHEAT;
        let age_5 = wheat
            .state_from_properties(&[("age", "5")])
            .expect("wheat age 5")
            .id;
        let mut exact = NbtCompound::new();
        exact.put_string("age", "5".to_string());
        assert!(state_matches(&exact, wheat, age_5));
        exact.put_int("age", 4);
        assert!(!state_matches(&exact, wheat, age_5));

        let mut range = NbtCompound::new();
        range.put_string("min", "3".to_string());
        range.put_string("max", "6".to_string());
        let mut ranged = NbtCompound::new();
        ranged.put_compound("age", range);
        assert!(state_matches(&ranged, wheat, age_5));

        let mut unknown = NbtCompound::new();
        unknown.put_string("facing", "north".to_string());
        assert!(!state_matches(&unknown, wheat, age_5));
    }

    #[test]
    fn nbt_contains_is_partial() {
        let mut actual = NbtCompound::new();
        actual.put_string("Lock", "key".to_string());
        actual.put_int("Other", 1);
        actual.put_list(
            "Items",
            vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3)],
        );
        let mut expected = NbtCompound::new();
        expected.put_string("Lock", "key".to_string());
        expected.put_list("Items", vec![NbtTag::Int(2)]);
        assert!(nbt_contains(
            &NbtTag::Compound(actual.clone()),
            &NbtTag::Compound(expected.clone())
        ));
        expected.put_int("Other", 2);
        assert!(!nbt_contains(
            &NbtTag::Compound(actual),
            &NbtTag::Compound(expected)
        ));
    }
}
