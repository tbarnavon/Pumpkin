//! Flammability overrides set by plugins, like Fabric's `FlammableBlockRegistry`: consulted before
//! the vanilla values in `Block.flammable`.

use std::sync::LazyLock;

use dashmap::DashMap;
use pumpkin_data::{Block, Flammable};

/// `None` makes the block fireproof.
static OVERRIDES: LazyLock<DashMap<u16, Option<(u8, u8)>>> = LazyLock::new(DashMap::new);

/// Sets the burn and spread chances of a block; both 0 makes it fireproof.
pub fn set(block: &Block, burn_chance: u8, spread_chance: u8) {
    let value = (burn_chance > 0 || spread_chance > 0).then_some((burn_chance, spread_chance));
    OVERRIDES.insert(block.id.as_u16(), value);
}

/// The block's flammability, with plugin overrides applied.
#[must_use]
pub fn get(block: &Block) -> Option<Flammable> {
    OVERRIDES.get(&block.id.as_u16()).map_or_else(
        || block.flammable.clone(),
        |value| {
            (*value).map(|(burn_chance, spread_chance)| Flammable {
                spread_chance,
                burn_chance,
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_replaces_and_clears_vanilla_values() {
        assert!(get(&Block::OAK_PLANKS).is_some());
        set(&Block::OAK_PLANKS, 0, 0);
        assert!(get(&Block::OAK_PLANKS).is_none());
        set(&Block::STONE, 5, 20);
        let stone = get(&Block::STONE).expect("stone is flammable now");
        assert_eq!((stone.burn_chance, stone.spread_chance), (5, 20));
    }
}
