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

## Protocol

Every serverbound and clientbound play packet was compared with 1.21.1's packet classes.
Where later versions changed a packet, the writers branch on the version. Some later packets
have no 1.21.1 counterpart, and the server sends what 1.21.1 uses instead:

| Later packet | 1.21.1 |
| --- | --- |
| `set_cursor_item` | `container_set_slot` for container -1, slot -1 |
| `set_player_inventory` | `container_set_slot` for container -2 |
| `entity_position_sync` | `teleport_entity` |
| `add_entity` for experience orbs | `add_experience_orb`, which carries the value |
| `pick_item_from_block` / `_entity` (serverbound) | `pick_item`, an inventory slot to swap into the hotbar |

**Item stacks** have their own codec for 1.20.5 to 1.21.1 (`data_component_v1_21.rs`),
written from 1.21.1's component stream codecs. It writes the tooltip flags, the older food,
tool, custom model data and potion layouts, and network NBT for components without a network
codec. Components 1.21.1 doesn't know are never sent. Components the server can't express in
1.21.1's form are left out of the stack: `can_place_on`, `can_break`, `instrument`,
`recipes`, `lock` and `pot_decorations`. Items from clients are read with the same codec.

**Particles** sent without options (commands, plugins) get default options in 1.21.1's
layouts, since the client can't decode an option particle without them.

## Worldgen

Density functions, Perlin noise and the aquifer run in f64, as in 1.21.1; splines stay f32 and
the end islands function stays float, as vanilla does. Climate values are cast to float before
quantizing. Surface rules use 1.21.1's positional randoms: big-endian MD5 seeds for vertical
gradients, and the legacy random for nether-style settings. Eroded badlands pillars are banded by
the surface rules after they are placed. Chunk generation is about 20 to 30% slower than with f32
(noise stage in a release bench: 149 ms to 195 ms).

## World files

Worlds open in vanilla 1.21.1, and vanilla 1.21.1 worlds open in Pumpkin (data versions 3953
to 3955 load; 3955 is written).
- **Layout:** the overworld in the world folder, the nether in `DIM-1`, the end in `DIM1`, saved
  data in `data/`, players in `playerdata/`, `advancements/` and `stats/`.
- **level.dat** holds what 1.21.1 reads there: `WorldGenSettings`, `GameRules` (string values),
  `DayTime` and `DragonFight`. Pumpkin's own data files are still written next to it.
- **Items** use 1.21.1's component forms: one-int custom model data, the potion id string, JSON
  text for names, lore and book pages, a plain book title. Components 1.21.1 lacks aren't saved.
- **Mobs** also save the `ArmorItems`, `HandItems` and `body_armor_item` lists, and load them
  when the later `equipment` compound is absent.
- **Offline players** get vanilla's UUID (`OfflinePlayer:<name>`, MD5), so their files carry
  over.
- **Advancements** are saved as `{criteria: {name: date}, done}` with a `DataVersion`.

## Tests

- **Worldgen fixtures:** the noise, surface and biome fixtures in `assets/tests/` are dumped from
  1.21.1's own generator by the Extractor, and the tests allow no mismatch.
- **Expected values:** tests that pinned 26.x ids now pin 1.21.1's.
- **Skipped:** the three Storage Drawers fixture tests are skipped; they need a 1.21.1 dump of
  the mod.

## Known gaps

- **Saddles:** pigs, striders and horses show a saddle through entity data in 1.21.1, not
  through the saddle equipment slot of later versions, which isn't sent. Saddled animals look
  unsaddled.
- **Unsent components:** see Protocol.
- **Content:** content added after 1.21.1 is removed (copper golem, creaking, happy ghast,
  nautilus, cushions, shelves, spears, ...). Where a mechanic differs, the code follows 1.21.1:
  - boats are one entity with a wood type;
  - thrown potions are one entity;
  - vehicles drop their item;
  - mob equipment tiers;
  - tempt range;
  - ender pearl and mace damage.
