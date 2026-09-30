use pumpkin_data::recipes::RecipeCategoryTypes;

use pumpkin_data::item::Item;
use pumpkin_data::tag::Taggable;

#[derive(Clone, Debug)]
pub enum OwnedRecipeIngredient {
    Simple(String),
    Tagged(String),
    OneOf(Vec<String>),
    /// Fabric `fabric:any_of`: matches if any part does.
    AnyOf(Vec<Self>),
    /// Fabric `fabric:all_of`: matches if every part does.
    AllOf(Vec<Self>),
    /// Fabric `fabric:difference`: matches `base` but not `subtracted`.
    Difference(Box<Self>, Box<Self>),
    /// Fabric `fabric:components`: `base` with required components. Pumpkin matches items, not
    /// stacks, so only `base` is checked.
    Components(Box<Self>),
}

/// The item's namespaced id: vanilla registry keys have no namespace, modded ones keep theirs.
fn item_id(item: &Item) -> String {
    if item.registry_key.contains(':') {
        item.registry_key.to_string()
    } else {
        format!("minecraft:{}", item.registry_key)
    }
}

impl OwnedRecipeIngredient {
    #[must_use]
    pub fn match_item(&self, item: &Item) -> bool {
        match self {
            Self::Simple(id) => item_id(item) == *id,
            Self::Tagged(tag) => item.is_tagged_with(tag).unwrap_or(false),
            Self::OneOf(ids) => ids.contains(&item_id(item)),
            Self::AnyOf(parts) => parts.iter().any(|part| part.match_item(item)),
            Self::AllOf(parts) => parts.iter().all(|part| part.match_item(item)),
            Self::Difference(base, subtracted) => {
                base.match_item(item) && !subtracted.match_item(item)
            }
            Self::Components(base) => base.match_item(item),
        }
    }

    /// Every item the ingredient matches, for recipe displays.
    #[must_use]
    pub fn matching_items(&self) -> Vec<&'static Item> {
        (0..Item::count())
            .filter_map(Item::from_id)
            .filter(|item| self.match_item(item))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct OwnedRecipeResult {
    pub item_id: String,
    pub count: u8,
    // TODO: Add components/enchantments if needed for the display result
}

#[derive(Clone, Debug)]
pub enum OwnedCraftingRecipe {
    Shaped {
        recipe_id: Option<String>,
        category: RecipeCategoryTypes,
        group: Option<String>,
        show_notification: bool,
        key: Vec<(char, OwnedRecipeIngredient)>,
        pattern: Vec<String>,
        result: OwnedRecipeResult,
    },
    Shapeless {
        recipe_id: Option<String>,
        category: RecipeCategoryTypes,
        group: Option<String>,
        ingredients: Vec<OwnedRecipeIngredient>,
        result: OwnedRecipeResult,
    },
}

#[derive(Clone, Debug)]
pub struct OwnedCookingRecipe {
    pub recipe_id: String,
    pub category: RecipeCategoryTypes,
    pub group: Option<String>,
    pub ingredient: OwnedRecipeIngredient,
    pub cooking_time: i32,
    pub experience: f32,
    pub result: OwnedRecipeResult,
}

#[derive(Clone, Debug)]
pub enum OwnedCookingRecipeType {
    Blasting(OwnedCookingRecipe),
    Smelting(OwnedCookingRecipe),
    Smoking(OwnedCookingRecipe),
    CampfireCooking(OwnedCookingRecipe),
}

#[derive(Clone, Debug)]
pub struct OwnedBrewingRecipe {
    pub recipe_id: String,
    pub input_item: String,
    pub input_potion: Option<String>,
    pub reagent: String,
    pub output_item: String,
    pub output_potion: Option<String>,
}

#[derive(Clone, Debug)]
pub enum DynamicRecipe {
    Crafting(OwnedCraftingRecipe),
    Cooking(OwnedCookingRecipeType),
    Brewing(OwnedBrewingRecipe),
}
