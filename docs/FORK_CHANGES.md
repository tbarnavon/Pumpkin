# Fork changes

What branch `modded` changes on top of upstream Pumpkin, by area: the commits, the files, and
why each change exists. The upstream sync (`ROADMAP.md`, item 11) reads this file to sort each
upstream merge into "drop ours", "adapt ours" or "unrelated".

- **Upstream:** `https://github.com/Pumpkin-MC/Pumpkin`, branch `master`.
- **Last sync:** merge base `4426d1113` (2026-09-28).
- **Fork commits since then:** 99 (2026-10-03, this file's commit included).

When you commit to `modded`, add the commit to its area here (or add an area). Hooks and host
functions also get a row in `HOOKS.md`. Paths below drop the `crates/` prefix; `WH` is
`pumpkin/src/plugin/loader/wasm/wasm_host/wit/v0_1`.

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
`pumpkin/src/block/registry.rs`, `pumpkin-world/src/block/mod.rs`, and two callers of changed
generated signatures: `pumpkin-world/src/generation/rule/block_match.rs`,
`pumpkin/src/entity/mob/enderman.rs`. Tests: `pumpkin-data/tests/dynamic_registries.rs`.

**Rebase note:** the `generated/` files come from codegen. On conflict, take upstream's codegen
inputs, keep the fork's codegen changes, and regenerate; don't merge generated files by hand.

## 2. Mod data dump

**Why:** the Extractor (a Fabric mod run once) dumps a mod's registries, tags, recipes and loot
tables; the server loads them at startup instead of running Java.

- `b2fdccf44` load Extractor mod dumps at startup
- `04df1ec41` refresh the Storage Drawers dump fixture

**Files:** `pumpkin-registry-ext/` (new crate, with the Storage Drawers dump as a test fixture),
`pumpkin/Cargo.toml`, `pumpkin/src/main.rs`,
`pumpkin/src/data/datapack/mod.rs`, `pumpkin/src/block/mod.rs`.

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
  for both loaders (`pumpkin-neoforge/` new crate, `pumpkin/src/net/java/loaders.rs` new,
  `pumpkin-config/src/modded.rs`, `clippy.toml` new, for the `NeoForge` doc word)
- `3035bfca5` NeoForge's synced config
- `b708340f8` Forge detection and the `forge:handshake` configuration tasks (`pumpkin-forge/`
  new crate, `pumpkin/src/net/java/loaders.rs`, `login/login_acknowledged.rs`,
  `pumpkin-config/src/modded.rs`)
- `a839b7299` modded menus on Forge clients (`WH/modded.rs`)
- `45dfd1177` mods' network channels per loader (`modded.register-loader-channels`; `WH/modded.rs`,
  `pumpkin/src/net/java/loaders.rs`, `pumpkin-forge/src/handshake.rs`)

**Files:** `pumpkin-fabric/` (new crate, tested against a payload a real Fabric server sent),
`pumpkin-protocol/src/codec/{modded_component,data_component}.rs`,
`pumpkin-protocol/src/java/client/{config,play}/update_tags.rs`,
`pumpkin/src/net/java/login/login_acknowledged.rs`, `pumpkin/src/net/java/pending.rs`,
`pumpkin/src/net/java/chunk_data/v1_18.rs`. Tests:
`pumpkin-protocol/tests/modded_component_stream_codec.rs`.

## 5. Recipes

**Why:** mod recipes come from the dump, and plugins look up and define recipes.

- `cfe0d95f9` parse tag ingredients written as `#tag` strings
- `b504d3619` match modded items in plugin and datapack recipe ingredients
- `2717a2ac3` Fabric custom ingredients (`fabric:all_of` and others)
- `981c23443` crafting handlers for recipes defined in code
- `f86ab3824` match crafting and cooking recipes from plugins
- `a490d007e` list crafting recipes by result

**Files:** `pumpkin/src/data/datapack/recipe_loader.rs`, `pumpkin-protocol/src/codec/recipe.rs`,
`pumpkin-protocol/src/java/client/play/recipe_book_add.rs`,
`pumpkin-inventory/src/crafting/{crafting_screen_handler,recipe_provider}.rs`,
`pumpkin/src/server/recipe.rs`, `WH/recipe.rs`. Tests: `pumpkin/tests/modded_recipe_book.rs`.

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
  `use-on-release`; `player.start-using-item`): `@C1@` (`entity/living.rs` use tick, finish and
  `clear_active_hand`; `item/{mod,registry}.rs`)
- Per-tick hooks batched into one `handle-tick-batch` call per plugin and tick: `1fa249829`
  (`WH/modded.rs` `PluginTickQueue`, flushed from `World::tick`; `entity/player.rs`)
- Menus: `cddec52a8`, `31377bbf9`
- Events: `b9be9ddc0`, `defeddb36`, `d719b608c`, `9bf64aded`, `729862d48`, `ada7f58d6`,
  `c19760f9c`, `e8ceb3e1e`, `d75320470`, `fcad77c09`
- Host functions: `bd7f4c35e`, `8c64bfb06`, `4c0cc6ae9`, `786ca3cd8`, `1d73c14ae`, `907bef135`
- Bulk world access: `5d96d025f`
- `f76a58841` a missing `must_use`
- Reorganisation (general functions out of `modded.wit` into core interfaces): `25b0b7b63`,
  `fb5343e2e`

**Files:** `pumpkin-plugin-wit/v0.1/*.wit` (`modded.wit` and `menu.wit` new),
`pumpkin-plugin-api/src/{modded,menu,crafting,lib}.rs` and `src/events/*`, `WH/*`,
`pumpkin-host-bindings/src/lib.rs`, `pumpkin/src/plugin/api/events/*`,
`pumpkin/src/plugin/login_queries.rs` (new), and the call sites the hooks and events fire from:
`pumpkin/src/block/{mod,registry}.rs`, `pumpkin/src/item/registry.rs`,
`pumpkin/src/entity/{player,item}.rs`, `pumpkin/src/net/java/play/{player_action,pick_item}.rs`,
`pumpkin/src/net/java/login/{encryption_response,plugin_response}.rs`,
`pumpkin/src/world/{mod,entity_tracker}.rs`, `pumpkin/src/block/blocks/{bed,straw_bed,shulker_box}.rs`,
`pumpkin/src/block/blocks/fire/fire.rs`, `pumpkin/src/block/fluid/lava.rs`,
`pumpkin/src/block/flammability.rs` (new), `pumpkin/src/lib.rs`,
`pumpkin-inventory/src/{screen_handler,generic_container_screen_handler}.rs`,
`pumpkin-data/src/data_component_impl/utility.rs`.

## 8. Host-held block data

**Why:** data first: hoppers, redstone and collisions run every tick and must not call the
plugin, so the host keeps the data and the plugin pushes changes. Saved in the block-entity NBT
under hidden keys (`PumpkinItemStorage`, `PumpkinSignals`, `PumpkinCollision`).

- `b9be9ddc0` item storages (hoppers), `acdbb3c6a` pooled slots
- `a6f9674ba` redstone and comparator output, `7b1acc125` per side
- `d36eaec65` per-position collision shapes

**Files:** `pumpkin/src/world/{item_storage,plugin_signals,plugin_shapes}.rs` (new),
`pumpkin/src/world/mod.rs`, `pumpkin/src/block/entities/hopper.rs`, `WH/world.rs`,
`WH/modded.rs`.

## 9. Plugin runtime

**Why:** a trapped plugin (a panic in WASM) shouldn't take its mod down until restart.

- `76482ef0b` restart a WASM plugin after it traps
- `81f35ff24` worker jobs (`scheduler.spawn-job`): `wasm_host/jobs.rs` (new),
  `wasm_host/mod.rs`, `WH/scheduler.rs`, `pumpkin/src/server/scheduler.rs`,
  `pumpkin-plugin-wit/v0.1/{scheduler,plugin}.wit`, `pumpkin-plugin-api/src/{lib,scheduler}.rs`
- `2d0f6aa31` drop an unused helper
- `38ea03a66` format fork-changed files
- `80a6603ee` `PLUGIN_API_VERSION` 2 to 3: native plugins see changed `StorageSlot`,
  `PluginSignals` and `set_plugin_redstone_output` (`pumpkin/src/plugin/mod.rs`)

**Files:** `pumpkin/src/plugin/loader/wasm/wasm_host/{mod,restart}.rs`, every `WH/*` resource
module, `pumpkin/src/plugin/mod.rs`, `pumpkin/src/server/scheduler.rs`.

## 10. Upstream bug fixes

**Why:** bugs found while testing that also affect upstream Pumpkin. These are candidates to
send upstream (`ROADMAP.md`, item 12); once merged there, drop them here.

- `6078f6e29` hoppers loaded from disk pushed down instead of their facing
  (`pumpkin/src/block/entities/hopper.rs`)
- `c0f42ce91` forced chunks never loaded (`pumpkin/src/world/{active_chunks,mod}.rs`)
- `f0b482006` player inventory slots synced by the wrong index (`WH/{modded,player}.rs`)
- `53edcddab` WIT sounds mapped to the wrong vanilla sounds (`WH/{modded,player,world}.rs`,
  `pumpkin-plugin-runtime/src/executor.rs`)
- `cfe0d95f9` tag ingredients written as `#tag` strings (also in area 5)
- `07ab1095f` configuration-phase kicks sent the reason as a string, not a text component, so the
  client couldn't read them (`pumpkin-protocol/src/java/client/config/config_disconnect.rs`)

## 11. Docs

Fork-only: `docs/DESIGN.md`, `docs/STATUS.md`, `docs/ROADMAP.md`, `docs/HOOKS.md`,
`docs/modloaders/` (one file per loader), this file.
`docs/FABRIC_API_PARITY.md` was removed on 2026-10-03; its open rows are in `HOOKS.md`, "Hook
catalog".

## Where rebases conflict

Files that both upstream and the fork edit often. Check these first after a sync:

- `pumpkin/src/world/mod.rs`: ticking, block entities, active chunks, collisions.
- `pumpkin/src/block/registry.rs`, `pumpkin/src/item/registry.rs`: hook dispatch.
- `pumpkin/src/entity/player.rs`: inventory tick, menus.
- `pumpkin/src/net/java/login/*`, `pending.rs`: the Fabric handshake.
- `pumpkin-data/src/generated/*`: regenerate, don't merge.
- `pumpkin-plugin-wit/v0.1/*.wit`: upstream mirrors the WIT to its own repo; keep fork additions
  additive.
