use std::fs;

use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;

/// Generates every recipe in the form `update_recipes` sends it to pre-1.21.2 clients.
pub fn build() -> TokenStream {
    let dir = std::path::Path::new("../../assets/datapack/data/minecraft/recipe");
    let mut paths: Vec<_> = fs::read_dir(dir)
        .expect("Missing recipe directory")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    // Same order as `RECIPES_CRAFTING`, whose index the recipe book uses as display id.
    paths.sort_by_key(|path| path.file_stem().unwrap().to_owned());
    let serializers: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../../assets/recipe_serializers.json").unwrap())
            .expect("Failed to parse recipe_serializers.json");

    let recipes = paths.iter().map(|path| {
        let id = format!("minecraft:{}", path.file_stem().unwrap().to_str().unwrap());
        let json: Value = serde_json::from_str(&fs::read_to_string(path).unwrap())
            .unwrap_or_else(|e| panic!("Failed to parse recipe {}: {e}", path.display()));
        let serializer = json["type"].as_str().unwrap();
        let serializer_id = serializers
            .iter()
            .position(|name| name == serializer)
            .unwrap_or_else(|| panic!("Unknown recipe serializer {serializer}"))
            as i32;
        let data = recipe_data(serializer, &json);
        quote! { SyncedRecipe { id: #id, serializer: #serializer_id, data: #data } }
    });

    quote! {
        /// One part of a recipe ingredient: an item or every item of a tag.
        #[derive(Clone, Copy, Debug)]
        pub enum SyncedIngredient {
            Item(&'static str),
            Tag(&'static str),
        }

        /// A recipe result without components.
        #[derive(Clone, Copy, Debug)]
        pub struct SyncedResult {
            pub id: &'static str,
            pub count: i32,
        }

        /// A recipe's serializer-specific fields, in network order.
        #[derive(Clone, Copy, Debug)]
        pub enum SyncedRecipeData {
            Shaped {
                group: &'static str,
                category: i32,
                width: i32,
                height: i32,
                /// Row-major; an empty ingredient is an empty slice.
                ingredients: &'static [&'static [SyncedIngredient]],
                result: SyncedResult,
                show_notification: bool,
            },
            Shapeless {
                group: &'static str,
                category: i32,
                ingredients: &'static [&'static [SyncedIngredient]],
                result: SyncedResult,
            },
            Cooking {
                group: &'static str,
                category: i32,
                ingredient: &'static [SyncedIngredient],
                result: SyncedResult,
                experience: f32,
                cooking_time: i32,
            },
            Stonecutting {
                group: &'static str,
                ingredient: &'static [SyncedIngredient],
                result: SyncedResult,
            },
            SmithingTransform {
                template: &'static [SyncedIngredient],
                base: &'static [SyncedIngredient],
                addition: &'static [SyncedIngredient],
                result: SyncedResult,
            },
            SmithingTrim {
                template: &'static [SyncedIngredient],
                base: &'static [SyncedIngredient],
                addition: &'static [SyncedIngredient],
            },
            /// Special crafting recipes only send their category.
            Special { category: i32 },
        }

        /// A recipe as `update_recipes` sends it.
        #[derive(Clone, Copy, Debug)]
        pub struct SyncedRecipe {
            pub id: &'static str,
            /// The network id of its `recipe_serializer`.
            pub serializer: i32,
            pub data: SyncedRecipeData,
        }

        /// Every recipe, ordered by id like `recipes::RECIPES_CRAFTING`.
        pub static SYNCED_RECIPES: &[SyncedRecipe] = &[#(#recipes),*];

        /// The `recipe_serializer` registry, by network id.
        pub static RECIPE_SERIALIZERS: &[&str] = &[#(#serializers),*];
    }
}

fn recipe_data(serializer: &str, json: &Value) -> TokenStream {
    let group = json["group"].as_str().unwrap_or("");
    match serializer {
        "minecraft:crafting_shaped" => {
            let category = crafting_category(json);
            let pattern: Vec<&str> = json["pattern"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row.as_str().unwrap())
                .collect();
            let rows = shrink(&pattern);
            let height = rows.len() as i32;
            let width = rows.first().map_or(0, |row| row.chars().count()) as i32;
            let ingredients = rows.iter().flat_map(|row| row.chars()).map(|symbol| {
                if symbol == ' ' {
                    quote! { &[] }
                } else {
                    ingredient(&json["key"][symbol.to_string()])
                }
            });
            let result = result(&json["result"]);
            let show_notification = json["show_notification"].as_bool().unwrap_or(true);
            quote! {
                SyncedRecipeData::Shaped {
                    group: #group,
                    category: #category,
                    width: #width,
                    height: #height,
                    ingredients: &[#(#ingredients),*],
                    result: #result,
                    show_notification: #show_notification,
                }
            }
        }
        "minecraft:crafting_shapeless" => {
            let category = crafting_category(json);
            let ingredients = json["ingredients"]
                .as_array()
                .unwrap()
                .iter()
                .map(ingredient);
            let result = result(&json["result"]);
            quote! {
                SyncedRecipeData::Shapeless {
                    group: #group,
                    category: #category,
                    ingredients: &[#(#ingredients),*],
                    result: #result,
                }
            }
        }
        "minecraft:smelting"
        | "minecraft:blasting"
        | "minecraft:smoking"
        | "minecraft:campfire_cooking" => {
            // CookingBookCategory: food, blocks, misc.
            let category = match json["category"].as_str().unwrap_or("misc") {
                "food" => 0,
                "blocks" => 1,
                _ => 2,
            };
            let default_time = if serializer == "minecraft:smelting" {
                200
            } else {
                100
            };
            let ingredient = ingredient(&json["ingredient"]);
            let result = result(&json["result"]);
            let experience = json["experience"].as_f64().unwrap_or(0.0) as f32;
            let cooking_time = json["cookingtime"].as_i64().unwrap_or(default_time) as i32;
            quote! {
                SyncedRecipeData::Cooking {
                    group: #group,
                    category: #category,
                    ingredient: #ingredient,
                    result: #result,
                    experience: #experience,
                    cooking_time: #cooking_time,
                }
            }
        }
        "minecraft:stonecutting" => {
            let ingredient = ingredient(&json["ingredient"]);
            let result = result(&json["result"]);
            quote! {
                SyncedRecipeData::Stonecutting { group: #group, ingredient: #ingredient, result: #result }
            }
        }
        "minecraft:smithing_transform" => {
            let template = ingredient(&json["template"]);
            let base = ingredient(&json["base"]);
            let addition = ingredient(&json["addition"]);
            let result = result(&json["result"]);
            quote! {
                SyncedRecipeData::SmithingTransform {
                    template: #template,
                    base: #base,
                    addition: #addition,
                    result: #result,
                }
            }
        }
        "minecraft:smithing_trim" => {
            let template = ingredient(&json["template"]);
            let base = ingredient(&json["base"]);
            let addition = ingredient(&json["addition"]);
            quote! {
                SyncedRecipeData::SmithingTrim { template: #template, base: #base, addition: #addition }
            }
        }
        _ => {
            let category = crafting_category(json);
            quote! { SyncedRecipeData::Special { category: #category } }
        }
    }
}

/// `CraftingBookCategory` network id: building, redstone, equipment, misc.
fn crafting_category(json: &Value) -> i32 {
    match json["category"].as_str().unwrap_or("misc") {
        "building" => 0,
        "redstone" => 1,
        "equipment" => 2,
        _ => 3,
    }
}

fn ingredient(json: &Value) -> TokenStream {
    let parts: Vec<&Value> = match json {
        Value::Array(values) => values.iter().collect(),
        value => vec![value],
    };
    let parts = parts.iter().map(|part| {
        if let Some(item) = part["item"].as_str() {
            quote! { SyncedIngredient::Item(#item) }
        } else if let Some(tag) = part["tag"].as_str() {
            quote! { SyncedIngredient::Tag(#tag) }
        } else {
            panic!("Unknown ingredient {part}")
        }
    });
    quote! { &[#(#parts),*] }
}

fn result(json: &Value) -> TokenStream {
    assert!(
        json.get("components").is_none(),
        "Recipe results with components are not synced: {json}"
    );
    let id = json["id"].as_str().unwrap();
    let count = json["count"].as_i64().unwrap_or(1) as i32;
    quote! { SyncedResult { id: #id, count: #count } }
}

/// Vanilla's `ShapedRecipePattern.shrink`: drops blank rows and columns around the pattern.
fn shrink<'a>(rows: &[&'a str]) -> Vec<&'a str> {
    let first = rows
        .iter()
        .map(|row| row.find(|c| c != ' ').unwrap_or(row.len()))
        .min()
        .unwrap_or(0);
    let last = rows.iter().filter_map(|row| row.rfind(|c| c != ' ')).max();
    let Some(last) = last else {
        return Vec::new();
    };
    let start = rows.iter().take_while(|row| row.trim().is_empty()).count();
    let end = rows.len()
        - rows
            .iter()
            .rev()
            .take_while(|row| row.trim().is_empty())
            .count();
    rows[start..end]
        .iter()
        .map(|row| &row[first..=last])
        .collect()
}
