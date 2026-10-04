# Branch `version-1.21.1`

This branch runs Pumpkin as a Minecraft 1.21.1 server (protocol 767). It sits on top of `modded`
and changes the data the server is generated from. The game code stays as close to `modded` as
possible, so syncs with `modded` mostly merge cleanly.

## The rule

What goes on the network is exactly 1.21.1: block states, items, entities, registries, tags,
packet ids and layouts. The game code was written against later versions, and it still uses some
of their concepts (data components, tags, game rules, chunk statuses, trades). Those are kept on
the server: either computed from 1.21.1's code by the Extractor, or copied from later data and
resolved against 1.21.1's ids. They are never sent to clients.

## Data

All assets come from the Extractor's `1.21.1` branch
(`./gradlew runServer -PextractExit`, output in `run/pumpkin_extractor_output`), except these
server-internal ones:

| Asset | What it is |
| --- | --- |
| `item_internal_components.json` | Components 1.21.1 defines in code (equippable, consumable, fuel, repairable, ...), from the Extractor |
| `brewing_recipes.json` | 1.21.1's code-defined brewing mixes, from the Extractor |
| `data_component_internal.json` | Later versions' component list; `is_networked()` is false for the ones 1.21.1 lacks |
| `packets_internal.json` | Later versions' packets; the ones 1.21.1 lacks get id -1 and are dropped on send |
| `tags_internal/` | Later tags the game code checks where 1.21.1 hard-codes the rule; a vanilla tag of the same name wins |
| `en_us_java_internal.json` | Later translation keys; sent with their English text as `fallback` |
| `chunk_status.json` | Pumpkin's chunk generation steps (later versions' list) |
| `datapack/.../trade_set`, `villager_trade`, `tags/villager_trade` | Later versions' data-driven trades (1.21.1's are code); trades of items 1.21.1 lacks are dropped |
| `datapack/.../world_preset/flat_all_dimensions.json` | Later preset Pumpkin supports |

Do not overwrite these with Extractor output. `entities.json` is also kept: its experience values
are random samples and change on every run.

## Codegen

`tools/pumpkin-codegen` reads 1.21.1's formats and rewrites them into the forms the game code
uses, for example:
- carvers, noise settings and density functions (`yScale`, `end_islands`, `weird_scaled_sampler`,
  the preliminary surface level);
- game rules: 1.21.1's ids under the code's later names, plus fixed internal rules;
- aliases for renamed blocks, property groups and enums (`LATER_*` lists in `block.rs`).
- what pre-1.21.2 clients need and later versions dropped: the full recipe list for
  `update_recipes` (`recipe_sync.rs`, numbered by `recipe_serializers.json`) and command
  argument ids (`command_argument_types.json`).

## Tests

- **Worldgen fixtures:** the fixtures in `assets/tests/` are translated from 26.x's ids to
  1.21.1's, by block name and state index and by biome name. End, nether and overworld terrain are
  the same in both versions, so the fixtures stay valid.
- **Expected values:** tests that pinned 26.x ids now pin 1.21.1's.
- **Skipped:** the three Storage Drawers fixture tests are skipped; they need a 1.21.1 dump of
  the mod.

## Known gaps

- **Density precision:** Pumpkin's density functions run in f32 (as in 26.x); 1.21.1 runs them in
  f64. Quantized climate values can be one off (under 1% of samples), so rare border blocks and
  biomes can differ from vanilla 1.21.1.
- **World files:** worlds are written with 26.x's data version and Pumpkin's chunk format, so a
  vanilla 1.21.1 server can't open them, and Pumpkin doesn't load vanilla 1.21.1 worlds.
- **Client check:** a real 1.21.1 client joins and plays (boats, inventories, enchanting,
  stonecutter, recipe book). Every serverbound packet was compared with 1.21.1's code;
  clientbound packets were only checked where something broke in game.
- **Pick block:** 1.21.1's `pick_item` (middle click in survival) has no handler yet.
- **Recipe book:** clicking a furnace recipe does nothing (also on later versions), and no ghost
  recipe is shown when ingredients are missing.
- **Content:** content added after 1.21.1 is removed (copper golem, creaking, happy ghast,
  nautilus, cushions, shelves, spears, ...). Where a mechanic differs, the code follows 1.21.1:
  - boats are one entity with a wood type;
  - thrown potions are one entity;
  - vehicles drop their item;
  - mob equipment tiers;
  - tempt range;
  - ender pearl and mace damage.
