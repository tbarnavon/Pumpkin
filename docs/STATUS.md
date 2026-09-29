# Status

Pins: Minecraft 26.3 (protocol 777), Fabric Loader 0.19.5, Fabric API 0.161.0+26.3,
Storage Drawers 26.3.0.1+fabric.

| Phase | State | Real-client result |
|:--|:--|:--|
| 0 Recon | done, see `DESIGN.md` | n/a (no code) |
| 1 Dynamic registries | done: overlay for blocks/states/items/tags/BE, entity, menu and component types; namespaced Anvil palettes; unknown blocks kept | not run (needs Phase 3 handshake) |
| 2 Data import | done: Extractor mod dump (`extractor/`, branch `modded-dump`) + `pumpkin-registry-ext` loader; recipes and loot tables merged into datapacks | server boots with Storage Drawers data; client join not expected to work before Phase 3 |
| 3 Fabric handshake | done: `pumpkin-fabric` (channel registration, `c:version`/`c:register`, registry sync byte-identical to Fabric's) | 2026-09-29: Fabric 26.3 client + Storage Drawers joins, crafts, places and breaks drawers (details below) |
| 4 Plugin API + Storage Drawers plugin | not started | not run |
| 5 Porting kit | not started | not run |

## Phase 1 tests

- `crates/pumpkin-data/tests/dynamic_registries.rs`: every vanilla state keeps its block and ID
  after mods are installed. Modded blocks, states, items and block-entity types are numbered
  directly after vanilla. Properties, shapes and tags (including additions to vanilla tags)
  resolve. A failed freeze installs nothing.
- `crates/pumpkin-world/tests/modded_chunk_roundtrip.rs`: an Anvil chunk with a modded block
  round-trips by namespaced name and properties. A block from a missing mod loads as the
  bedrock stand-in and is written back unchanged, and it is dropped once the stand-in is replaced.
- Existing suites still pass: `cargo test -p pumpkin-data -p pumpkin-world -p pumpkin`. Clippy is
  clean on the touched crates.
- Not yet verified with a real client: that needs the Phase 2 dump loader and the Phase 3
  handshake.

## Phase 2

- Dump: `source tools/gradle-env.sh && cd extractor && ./gradlew runModDump -PmodDumpNamespaces=storagedrawers`.
  It writes `run-moddump/pumpkin_extractor_output/mods/<ns>.json` and `fabric_registry_sync.json`,
  the exact map a real Fabric server sends.
- Install: copy `mods/<ns>.json` into `<server>/mod-data/`. On startup Pumpkin checks that every
  ID matches the dump and refuses to start on a mismatch.
- Storage Drawers 26.3.0.1: 150 blocks, 965 states, 166 items, 9 block-entity types, 6 menus,
  5 component types. The IDs match the reference server; covered by
  `crates/pumpkin-registry-ext/tests/storage_drawers_dump.rs`.
- Boot test (2026-09-29): 127 of 132 recipes imported. 5 were skipped because they use Storage
  Drawers' custom recipe serializers (Phase 4). All 15 loot tables were imported.
- Block drops now look up `<namespace>:blocks/<path>`, as vanilla `Block.getLootTable` does.

## Phase 3 (real client, 2026-09-29)

Fabric Loader 0.19.5, Fabric API 0.161.0+26.3, Storage Drawers 26.3.0.1 and JEI, against a debug
build of the fork.

Works:
- Join. The handshake, registry sync and configuration complete, and the world loads.
- Crafting a drawer from the imported recipes.
- Placing and breaking drawers.

Doesn't work yet (all Storage Drawers server logic, which is Phase 4):
- Drawers always face the default direction. Placement doesn't use the mod's facing logic.
- Drawers don't drop when broken. They have no loot table; the mod drops them from
  `BlockDrawers.getDrops`.
- Items can't be inserted or extracted, and there are no GUIs or counts.

Issues found and fixed on the way:
- `recipe_book_add` failed to decode. Tag ingredients written as `"#ns:path"` strings were parsed
  as items and encoded as air, which the client rejects. This bug exists in upstream Pumpkin too.
  Found by decoding Pumpkin's packet with Minecraft's codec (Extractor `-Dpumpkin.checkPackets`).
- Recipes use Fabric `c:` convention tags, so the dump now includes every non-`minecraft` tag.

Open:
- One rejoin was kicked for exceeding 500 packets/s; the user suspects JEI. The kick log line now
  includes the packet ID so the next occurrence can be identified.
- World generation is slow. That's partly the debug build. A fast path now avoids hashing tag names
  in `has_tag` when mods are loaded (clippy and tests pass).


## Phase 4, stage 1: standard drawers

The Storage Drawers server logic lives in `../storagedrawers-plugin` (WASM, `wasm32-wasip2`),
ported from the decompiled 26.3.0.1 jar. The plugin loads on the release build and registers all
78 standard drawer blocks (13 woods including framed, full or half, 1/2/4 slots).

Implemented:
- Facing on placement (`getStateForPlacement`), and offhand keys on placement (`setPlacedBy`).
- Right-click to insert, and a double right-click within 10 ticks to insert the whole inventory.
- Left-click to take one item, or a stack with shift. In creative, hitting a face slot doesn't
  break the drawer.
- Upgrades added by right-click, with SD's group, multiple and one-stack rules. The storage
  multipliers, void, creative storage and vending, and balance fill all work.
- Drawer, quantify, shroud, suspend and priority keys (sneak-use cycles priority), keyrings, and
  personal key ownership.
- `dropMode` KEEP, where the drop carries `block_entity_data` and `max_stack_size` 1, and DROP.
- Block-entity NBT in the mod's exact layout. The `Upgrades` list is always written, as vanilla's
  `TagValueOutput.list` does.
- Config `plugins/data/storagedrawers/storagedrawers-common.toml` with SD's defaults.

Real client (2026-09-29, `fast` build, fresh world):
- Taking one item with left-click, and a stack with shift + left-click, works.
- Breaking a filled drawer in survival drops it with its contents, and placing it back restores
  them.
- The earlier "no drops" report was a plugin trap: every plugin sound lookup trapped, which left
  the plugin failed until restart. Fixed by mapping WIT sounds to vanilla sounds by name.

Host API added for this: `give-item`, `drop-item`, and item <-> NBT conversion. The plugin
inventory setters sent `ContainerSetSlot` with inventory indexes, which desynced the hotbar. That
bug is also upstream, and it's fixed.

Not yet: the drawer GUI, compacting drawers, controller and I/O, hopper/magnet/redstone, framing
table, detached drawers, and remote upgrades. The GUI needs two host features first:
plugin-defined menus (opened with `fabric-menu-api-v1:open_screen`), and modded item components
such as `storagedrawers:drawer_count`, which Pumpkin currently drops.

## Phase 4: plugin restart after a trap

A trap used to leave a plugin's store failed until the server restarted. Now a supervisor waits
on each store driver and, when one fails, cancels the plugin's tasks, drops its event handlers and
commands, instantiates it again from the cached component, and runs `on_load` on the new
instance. Block and item hooks stay registered, because they point at the plugin rather than at
an instance. In-memory plugin state is lost. After 3 restarts within 60 s the plugin stays down.

Real client (2026-09-29): a test build of the Storage Drawers plugin panicked when a drawer was
used with a stick. The log showed the trap, `Wasm plugin trapped, restarting it` and
`Restarted Wasm plugin` in the same second, and drawers kept working right after.

Not handled: permissions a plugin registers in `on_load` can't be removed, so registering them
again fails after a restart. Upstream hot reload has the same problem.

## Phase 4: drawer GUI

Host:
- `ItemStack` keeps component types Pumpkin has no implementation of (modded ones) as raw NBT, by
  raw id in `minecraft:data_component_type`: saved under their name, and sent as a network NBT
  tag, which is vanilla's default component stream codec. Tested in
  `pumpkin-protocol/tests/modded_item_components.rs` against the expected bytes.
- Core `menu` interface: the plugin lists the slots (the viewer's inventory, or its own), and the
  host runs the vanilla click logic, asking the plugin through `handle-menu-call` for its slots'
  contents, `set`, `mayPlace`, `mayPickup`, max count and the shift-click ranges.
- `modded.open-menu`: opens a modded menu type with `fabric-menu-api-v1:open_screen`
  (`Networking.OpenScreenPayload`: type id, container id byte, title, the type's data).

Plugin: `ContainerDrawers1/2/4`, `SlotDrawer` (counts of 0 or 128+ in `drawer_count`),
`SlotUpgrade` with `DrawerUpgradeData`'s add/remove/swap rules, `quickMoveStack`.

Real client (2026-09-30): sneak + empty hand opens the drawer UI; it works.

Not verified: the menu closing by distance (cannot move with a menu open), and a packet
comparison of `open_screen` against a real Fabric server.

