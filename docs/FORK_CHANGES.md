# Fork changes

What branch `latest` changes on top of upstream Pumpkin, by area: the commits, the files, and
why each change exists. The upstream sync (`ROADMAP.md`, item 11) reads this file to sort each
upstream merge into "drop ours", "adapt ours" or "unrelated".

- **Upstream:** `https://github.com/Pumpkin-MC/Pumpkin`, branch `master`.
- **Last sync:** merge base `4426d1113` (2026-09-28).
- **Fork commits since then:** 99 (2026-10-03, this file's commit included).

When you commit to `latest`, add the commit to its area here (or add an area). Hooks and host
functions also get a row in `HOOKS.md`. Paths below drop the `crates/` prefix; `WH` is
`pumpkin-wasm-host-v0_1/src`.

Upstream split the server crate (`1859221e7`): `pumpkin-core` (the server), `pumpkin` (the bin),
`pumpkin-wasm-host` (the loader: instantiation, restarts, worker jobs, task delivery),
`pumpkin-wasm-host-common` (what every API version shares) and `pumpkin-wasm-host-v0_{1,2}`.
Core can't name Wasm host types, so it calls plugin content through traits in
`pumpkin-core/src/plugin/modded.rs` (`PluginBlockHooks`, `PluginItemHooks` via
`ItemBehaviour::plugin_hooks`, `PluginTickTarget`, the data-only `PluginTickQueue`,
`ANY_TICKER`) and `ScreenHandler::checks_can_use_each_tick` for plugin menus.

Upstream branched a `pumpkin:plugin@0.2.0` API from v0.1 (`361c34c4d`). Every fork addition to
the v0.1 WIT and host is mirrored in `pumpkin-plugin-wit/v0.2` and `pumpkin-wasm-host-v0_2`: the
v0.2 crate is v0.1's with upstream's renames (`v0_1` to `v0_2`, `@0.1.0` to `@0.2.0`, `_v0_2` on
`PluginHostState` methods), so add to v0.1 and mirror. Since `8adc29feb` v0.2 also has
upstream's v0.2-only API (GameTest: `pumpkin-plugin-wit/v0.2/gametest.wit`,
`pumpkin-wasm-host-v0_2/src/gametest.rs`), so mirror by hand; regenerating v0_2 from v0_1 would
drop it. Block, item and menu behaviour
(`PluginBlock`, `PluginItem`, `PluginMenuHandler`) is shared by both versions in
`pumpkin-wasm-host-common/src/modded/`; `WH/{modded,menu}.rs` only build each version's calls,
behind the `GuestCalls` table each version exports as `GUEST`.

## 1. Modded registries at runtime

**Why:** mod blocks, block states, items, block-entity types and tags come from the mod data
dump at startup; Pumpkin's registries are generated at compile time, so the fork adds a runtime
overlay after the vanilla ids.

- `7c587960c` runtime overlay for modded registry entries
- `c61ef77d5` skip tag lookups for ids no mod added to a tag (perf)
- `c649d737e` send the block-entity type of `block_entity_data` components

**Files:** `pumpkin-data/src/dynamic/*` (new), `pumpkin-data/src/{blocks,block_state,lib}.rs`,
`pumpkin-data/src/generated/{block,item,tag,fluid,screen,flower_pot_transformations}.rs`,
`tools/pumpkin-codegen/src/{block,item,tag,screen,flower_pot_transformations}.rs`,
`pumpkin-core/src/block/registry.rs`, `pumpkin-world/src/block/mod.rs`, and two callers of changed
generated signatures: `pumpkin-world/src/generation/rule/block_match.rs`,
`pumpkin-core/src/entity/mob/enderman.rs`. Tests: `pumpkin-data/tests/dynamic_registries.rs`.

**Rebase note:** the `generated/` files come from codegen. On conflict, take upstream's codegen
inputs, keep the fork's codegen changes, and regenerate; don't merge generated files by hand.

## 2. Mod data dump

**Why:** the Extractor (a Fabric mod run once) dumps a mod's registries, tags, recipes and loot
tables; the server loads them at startup instead of running Java.

- `b2fdccf44` load Extractor mod dumps at startup
- `04df1ec41` refresh the Storage Drawers dump fixture

**Files:** `pumpkin-registry-ext/` (new crate, with the Storage Drawers dump as a test fixture),
`pumpkin/Cargo.toml`, `pumpkin-core/Cargo.toml`, `pumpkin/src/main.rs`,
`pumpkin-core/src/data/datapack/mod.rs`, `pumpkin-core/src/block/mod.rs`.

## 3. Saving modded blocks and items

**Why:** worlds must keep modded blocks and items across restarts, and keep unknown ones rather
than delete them.

- `0c5b875f4` save modded blocks by namespaced name, keep unknown blocks
- `ab6df3493` save modded item ids, keep stacks with unknown components
- `b45e6363d` keep modded item components as raw NBT

**Files:** `pumpkin-world/src/chunk/{format/mod.rs,format/unknown_blocks.rs,mod.rs,palette.rs}`,
`pumpkin-world/src/chunk_system/chunk_state.rs`, `pumpkin-data/src/item_stack/mod.rs`,
`pumpkin-protocol/src/codec/item_stack_seralizer.rs`. Tests:
`pumpkin-world/tests/modded_chunk_roundtrip.rs`, `pumpkin-protocol/tests/modded_item_components.rs`.

## 4. Mod loader client networking

**Why:** a modded client checks the server's registries and channels during configuration, and
reads modded components with the mod's own stream codecs. This is the per-loader part the
project must support.

- `4843eeb1c` Fabric configuration handshake and registry sync
- `728d5c4e5` stream codecs for modded component types
- `a904c6276` NeoForge detection, channel negotiation and registry sync; one detection step
  for both loaders (`pumpkin-neoforge/` new crate, `pumpkin-core/src/net/java/loaders.rs` new,
  `pumpkin-config/src/modded.rs`, `clippy.toml` new, for the `NeoForge` doc word)
- `3035bfca5` NeoForge's synced config
- `b708340f8` Forge detection and the `forge:handshake` configuration tasks (`pumpkin-forge/`
  new crate, `pumpkin-core/src/net/java/loaders.rs`, `login/login_acknowledged.rs`,
  `pumpkin-config/src/modded.rs`)
- `a839b7299` modded menus on Forge clients (`WH/modded.rs`)
- `45dfd1177` mods' network channels per loader (`modded.register-loader-channels`; `WH/modded.rs`,
  `pumpkin-core/src/net/java/loaders.rs`, `pumpkin-forge/src/handshake.rs`)

**Files:** `pumpkin-fabric/` (new crate, tested against a payload a real Fabric server sent),
`pumpkin-protocol/src/codec/{modded_component,data_component}.rs`,
`pumpkin-protocol/src/java/client/{config,play}/update_tags.rs`,
`pumpkin-core/src/net/java/login/login_acknowledged.rs`, `pumpkin-core/src/net/java/pending.rs`,
`pumpkin-core/src/net/java/chunk_data/v1_18.rs`. Tests:
`pumpkin-protocol/tests/modded_component_stream_codec.rs`.

## 5. Recipes

**Why:** mod recipes come from the dump, and plugins look up and define recipes.

- `cfe0d95f9` parse tag ingredients written as `#tag` strings
- `b504d3619` match modded items in plugin and datapack recipe ingredients
- `2717a2ac3` Fabric custom ingredients (`fabric:all_of` and others)
- `981c23443` crafting handlers for recipes defined in code
- `f86ab3824` match crafting and cooking recipes from plugins
- `a490d007e` list crafting recipes by result

**Files:** `pumpkin-core/src/data/datapack/recipe_loader.rs`, `pumpkin-protocol/src/codec/recipe.rs`,
`pumpkin-protocol/src/java/client/play/recipe_book_add.rs`,
`pumpkin-inventory/src/crafting/{crafting_screen_handler,recipe_provider}.rs`,
`pumpkin-core/src/server/recipe.rs`, `WH/recipe.rs`. Tests: `pumpkin-core/tests/modded_recipe_book.rs`.

## 6. Config

**Why:** modded content can be switched on and off; Bedrock clients can't show it.

- `7fbd403c3` `[modded]` config section
- `6ac0809a6` turn Bedrock off while modded content is enabled

**Files:** `pumpkin-config/src/modded.rs` (new), `pumpkin-config/src/lib.rs`,
`pumpkin/src/main.rs`.

## 7. Plugin API: hooks, events, host functions

**Why:** a mod's server side is rewritten as a WASM plugin; these are the hook points and
functions it needs. Each one is listed in `HOOKS.md`.

- Block and item hooks: `add68fe15`, `0084f1557`, `e51c7666c`, `95105a9ab`, `342715c3c`,
  `044588b3a`, `c3b1024a1`, `dfe6c259c`, `13b9d100c`
- Item use over time (`finish-using`, `release-using`, `use-tick`, `stop-using`,
  `use-on-release`; `player.start-using-item`): `d4ab33e9a` (`entity/living.rs` use tick, finish and
  `clear_active_hand`; `item/{mod,registry}.rs`)
- Per-tick hooks batched into one `handle-tick-batch` call per plugin and tick: `1fa249829`
  (`WH/modded.rs` `PluginTickQueue`, flushed from `World::tick`; `entity/player.rs`)
- Menus: `cddec52a8`, `31377bbf9`
- Events: `b9be9ddc0`, `defeddb36`, `d719b608c`, `9bf64aded`, `729862d48`, `ada7f58d6`,
  `c19760f9c`, `e8ceb3e1e`, `d75320470`, `fcad77c09`
- Death and drops events (`living-death-event`, `living-drops-event`): `90e30cf06`
  (`entity/living.rs` `allow_death`, `on_death`: loot and equipment collected before dropping)
- Host functions: `bd7f4c35e`, `8c64bfb06`, `4c0cc6ae9`, `786ca3cd8`, `1d73c14ae`, `907bef135`
- Bulk world access: `5d96d025f`
- `f76a58841` a missing `must_use`
- Reorganisation (general functions out of `modded.wit` into core interfaces): `25b0b7b63`,
  `fb5343e2e`

**Files:** `pumpkin-plugin-wit/v0.{1,2}/*.wit` (`modded.wit` and `menu.wit` new),
`pumpkin-plugin-api/src/{modded,menu,crafting,lib}.rs` and `src/events/*`, `WH/*` and its v0_2
mirror, `pumpkin-wasm-host-common/src/modded/*`, `pumpkin-core/src/plugin/modded.rs`, `pumpkin-wasm-host-v0_{1,2}/src/bindings.rs`, `pumpkin-core/src/plugin/api/events/*`,
`pumpkin-core/src/plugin/login_queries.rs` (new), and the call sites the hooks and events fire from:
`pumpkin-core/src/block/{mod,registry}.rs`, `pumpkin-core/src/item/registry.rs`,
`pumpkin-core/src/entity/{player,item}.rs`, `pumpkin-core/src/net/java/play/{player_action,pick_item}.rs`,
`pumpkin-core/src/net/java/login/{encryption_response,plugin_response}.rs`,
`pumpkin-core/src/world/{mod,entity_tracker}.rs`, `pumpkin-core/src/block/blocks/{bed,straw_bed,shulker_box}.rs`,
`pumpkin-core/src/block/blocks/fire/fire.rs`, `pumpkin-core/src/block/fluid/lava.rs`,
`pumpkin-core/src/block/flammability.rs` (new), `pumpkin-core/src/lib.rs`,
`pumpkin-inventory/src/{screen_handler,generic_container_screen_handler}.rs`,
`pumpkin-data/src/data_component_impl/utility.rs`.

## 8. Host-held block data

**Why:** data first: hoppers, redstone and collisions run every tick and must not call the
plugin, so the host keeps the data and the plugin pushes changes. Saved in the block-entity NBT
under hidden keys (`PumpkinItemStorage`, `PumpkinSignals`, `PumpkinCollision`).

- `b9be9ddc0` item storages (hoppers), `acdbb3c6a` pooled slots
- `a6f9674ba` redstone and comparator output, `7b1acc125` per side
- `d36eaec65` per-position collision shapes

**Files:** `pumpkin-core/src/world/{item_storage,plugin_signals,plugin_shapes}.rs` (new),
`pumpkin-core/src/world/mod.rs`, `pumpkin-core/src/block/entities/hopper.rs`, `WH/world.rs`,
`WH/modded.rs`.

## 9. Plugin runtime

**Why:** a trapped plugin (a panic in WASM) shouldn't take its mod down until restart.

- `76482ef0b` restart a WASM plugin after it traps
- `81f35ff24` worker jobs (`scheduler.spawn-job`): `pumpkin-wasm-host/src/jobs.rs` and `pumpkin-wasm-host-common/src/jobs.rs` (new),
  `pumpkin-wasm-host/src/runtime.rs`, `WH/scheduler.rs`, `pumpkin-wasm-host-common/src/scheduler.rs`, `pumpkin-wasm-host/src/scheduler.rs`,
  `pumpkin-plugin-wit/v0.1/{scheduler,plugin}.wit`, `pumpkin-plugin-api/src/{lib,scheduler}.rs`
- `2d0f6aa31` drop an unused helper
- `38ea03a66` format fork-changed files
- `80a6603ee` `PLUGIN_API_VERSION` 2 to 3: native plugins see changed `StorageSlot`,
  `PluginSignals` and `set_plugin_redstone_output` (`pumpkin-core/src/plugin/mod.rs`)

**Files:** `pumpkin-wasm-host/src/{runtime,restart}.rs`, `pumpkin-wasm-host-common/src/plugin.rs`, every `WH/*` resource
module, `pumpkin-core/src/plugin/mod.rs`, `pumpkin-wasm-host-common/src/scheduler.rs`, `pumpkin-wasm-host/src/scheduler.rs`.

## 10. Upstream bug fixes

**Why:** bugs found while testing that also affect upstream Pumpkin. These are candidates to
send upstream (`ROADMAP.md`, item 12); once merged there, drop them here.

- `6078f6e29` hoppers loaded from disk pushed down instead of their facing
  (`pumpkin-core/src/block/entities/hopper.rs`)
- `c0f42ce91` forced chunks never loaded (`pumpkin-core/src/world/{active_chunks,mod}.rs`)
- `f0b482006` player inventory slots synced by the wrong index (`WH/{modded,player}.rs`)
- `53edcddab` WIT sounds mapped to the wrong vanilla sounds (`WH/{modded,player,world}.rs`,
  `pumpkin-plugin-runtime/src/executor.rs`)
- `cfe0d95f9` tag ingredients written as `#tag` strings (also in area 5)
- `07ab1095f` configuration-phase kicks sent the reason as a string, not a text component, so the
  client couldn't read them (`pumpkin-protocol/src/java/client/config/config_disconnect.rs`)
- `523f5c9a6` arrows passed their shooter as the direct entity, and the damage event packet sent
  cause and direct entity swapped (`pumpkin-core/src/entity/{projectile/arrow,living}.rs`)
- `1d3b8e6dc` `setblock` and `fill` read only a block id, not `[properties]{nbt}`
  (`pumpkin-command/src/argument_types/{block,block_predicate}.rs`,
  `pumpkin-core/src/command/commands/{setblock,fill}.rs`)
- `96e8e067c` new worlds kept the placeholder spawn height, so the console and other spawn users
  sat at Y 200 (`pumpkin-core/src/server/mod.rs`)
- `eae26f083` adventure mode ignored `can_break` and `can_place_on`: any block broke, none placed
  (`pumpkin-core/src/entity/player/adventure.rs` new, `pumpkin-core/src/net/java/play/{player_action,use_item_on}.rs`,
  `pumpkin-core/src/block/registry.rs`)
- `dbc56b4ed` completing a villager trade deadlocked the server: the trade callback
  locked the merchant screen handler it ran under (`pumpkin-core/src/entity/passive/villager/mod.rs`)
- `22f9cb735` a right click interacted twice (`interact_at` and `interact`), so
  players mounted twice and items were used twice (`pumpkin-core/src/net/java/play/interact.rs`,
  `pumpkin-core/src/entity/mod.rs`)
- `ef539eaf1` releasing shift was handled as pressing it
  (`pumpkin-core/src/net/java/play/player_command.rs`)
- `b2a0f0534` decorated pots lost their sherds (`pot_decorations` kept no data)
  (`pumpkin-data/src/data_component_impl/basic.rs`, `pumpkin-core/src/block/entities/decorated_pot.rs`,
  `pumpkin-protocol/src/codec/data_component.rs`, `tools/pumpkin-codegen/src/item.rs`)
- `5aa0f45ce` goat horns always played Ponder: `instrument` kept no data and was sent as an empty
  inline instrument (`pumpkin-data/src/data_component_impl/basic.rs`,
  `pumpkin-core/src/item/items/goat_horn.rs`, `pumpkin-protocol/src/codec/data_component.rs`)

## 11. Docs

Fork-only: `docs/DESIGN.md`, `docs/STATUS.md`, `docs/ROADMAP.md`, `docs/HOOKS.md`,
`docs/modloaders/` (one file per loader), this file.
`docs/FABRIC_API_PARITY.md` was removed on 2026-10-03; its open rows are in `HOOKS.md`, "Hook
catalog".

## Where rebases conflict

Files that both upstream and the fork edit often. Check these first after a sync:

- `pumpkin-core/src/world/mod.rs`: ticking, block entities, active chunks, collisions.
- `pumpkin-core/src/block/registry.rs`, `pumpkin-core/src/item/registry.rs`: hook dispatch.
- `pumpkin-core/src/entity/player.rs`: inventory tick, menus.
- `pumpkin-core/src/net/java/login/*`, `pending.rs`: the Fabric handshake.
- `pumpkin-data/src/generated/*`: regenerate, don't merge.
- `pumpkin-plugin-wit/v0.{1,2}/*.wit`: upstream mirrors the WIT to its own repo; keep fork
  additions additive, and in both versions.
