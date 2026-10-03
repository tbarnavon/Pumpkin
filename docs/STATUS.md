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


## Phase 4: the rest of Storage Drawers (plugin only, 2026-09-30)

Plugin commits 03e454a..c8320db (plus a clippy clean-up), no host change. Build and clippy are
clean, and the release plugin loads (86 drawer blocks: 78 standard, 8 compacting).

Implemented, each from the 26.3.0.1 source:
- Controller hopper storage and controller I/O (`DrawerStorageImpl` over `drawerSlots`). The
  controller and every I/O block its search binds get a host storage mirroring the network's
  slots in priority order, so hoppers fill populated drawers first. The cache refreshes on
  `BlockController.tick` (100 ticks, persisted tick chain) and on use, and follows every drawer
  save. This fixes "a hopper can't insert into the controller".
- Fixes: a drawer's own hopper slots now follow `getAccessibleDrawerSlots` (populated slots first),
  hopper moves rebalance balanced drawers, and inserts and takes rebalance across the network
  (`StorageUtil.onNetwork`).
- Redstone upgrade (combined, min, max) and comparator output.
- Remote and remote group upgrades: binding on the controller front, `checkBoundController`,
  `RemoteNodes`, remote search roots with `maxRange` / `maxGroupRange`, unbinding when the
  controller goes, and the 60-tick check of held bound upgrades.
- Compacting drawers (2 and 3 slots, full, half, framed): pooled storage and NBT, compacting chains
  from 3x3 / 2x2 recipes both ways through `match-crafting`, `CompTierRegistry` rules, the `slots`
  block state, and the comp UIs.
- Detached drawers: drawer puller, putting drawers back, `forceMaxCapacityCheck`, heavy drawers
  (slowness) with the portability upgrade, and the filled-drawer storage deny rule.
- Key buttons, keyring sneak-use rotation, and the keyring cooldown.
- Special recipes: add_upgrade, add_detached_upgrade, keyring, personal_key_cycle,
  remote_group_upgrade.
- Framing table (two-part block, UI, taking framed blocks apart), framed drawers, compacting
  drawers, trims, controllers and I/O keeping their materials on place, drop and pick, and
  retrim / repartition with sneak-use.
- Conversion upgrade: `itemEquivalenceGroups`.

Blocked on a missing host API:
- **Hopper and magnet upgrades.** `BlockEntityDrawers.addItemEntity` (from `BlockDrawers.tick` /
  `pushItemsTick` / `suckInItems`, and `entityInside` for the hopper upgrade) reads an
  `ItemEntity`'s stack, puts it into the drawer, then shrinks or discards the entity. The plugin
  needs to read an item entity's stack (item, count, components) and set or remove it, for
  entities found with `world.get-entities-in-box`. Nothing gives a plugin an item entity's stack:
  `entity` has no item accessor, and `item-spawn-event` carries only the item name.
- **Keyring insert and remove in the inventory.** `ItemKeyring.overrideOtherStackedOnMe`
  right-click puts the cursor key into the keyring, or takes the first key out into the cursor.
  The plugin needs to set the player's cursor (carried) item, or a stacked-on-me item hook. The
  `inventory-click-event` can cancel a click but cannot change the cursor.
- **Keyring contents when it burns.** `ItemKeyring.onDestroyed` spills its keys; there is no hook
  for an item entity being destroyed.
- **`canStoreInContainers`** (Filled and Detached): `Item.canFitInsideContainerItems` keeps filled
  drawers out of bundles and shulker boxes. No per-item hook in the host, so they are allowed.
- **Items with `frame_data` or `controller_binding`** (framing table, framed drawers and blocks,
  bound remote upgrades). Both components have their own stream codec (`FrameData.STREAM_CODEC`:
  four `ItemStack.OPTIONAL_STREAM_CODEC`; `ControllerBinding.STREAM_CODEC`: bool + three ints),
  but the host sends every modded component as network NBT. Any packet carrying such a stack
  fails to decode on the client, which disconnects ("Failed to decode packet
  container_set_content" when opening the framing table). The host needs to encode modded
  components with the mod's stream codec (a codec description in the mod dump, or a plugin
  encoder). Seen 2026-09-30; left as is by the user's choice.
- **Conversion upgrade tags.** `ItemStackTagMatcher` matches items sharing an allowed item tag
  (`oreTypeAllowList` x `oreMaterialAllowList`, `tagAllowList`); the host exposes no item tags.
  Equivalence groups work.

Known limits (worked around, not blocking):
- Strong redstone power: the host gives one strong power to all sides; SD powers only the block
  below, so none is given (only matters with `analogOutput = false`).
- Compacting drawers through hoppers: host slots are independent, so several hoppers on one
  compacting drawer in the same tick can take a little more or less than the pool holds; an
  insert that doesn't fit the pool is lost.
- Controllers after a restart serve hoppers from their stored storage until their first tick;
  a change before that is diffed against the drawers' saved state.
- `findLowerTier` starts from the 1x1 craft (the host can't list recipes); a shapeless 2x2 / 3x3
  recipe counts where SD needs a shaped one.

Real-client results (2026-09-30):
- Work: hoppers into and out of drawers and the controller, controller I/O (after the refresh
  fix), left-click slot choice (after the eye ray-trace fix), controller right-click and
  double-click, redstone upgrade and comparator, compacting drawers (UI, hoppers), drawer puller
  and putting drawers back, add_detached_upgrade (as in single player: the capacity only matters
  with `forceMaxCapacityCheck`), key buttons, keyring rotation, add_upgrade and keyring recipes,
  conversion upgrade with an `itemEquivalenceGroups` entry.
- Fixed while testing: controllers placed by an older build had no block entity; the attack hit
  point is the block centre, so the plugin ray-traces from the eyes; a lost scheduled tick left the
  controller refresh stopped.
- Fails: framing table (disconnect, see the `frame_data` entry above).

Real-client tests to run:
1. Hopper into a controller and into a controller I/O: items go to drawers holding them first,
   then empty ones; hopper under the controller pulls items out; counts update on the drawers.
2. Right-click the controller front with an item no drawer holds: it goes into an empty drawer
   (`allowEmpty`); double-click puts only items already stored.
3. Redstone upgrade on a drawer with a comparator next to it.
4. Remote upgrade: bind on the controller front, put it in a drawer away from the network, check
   the controller reaches it; break the controller, the upgrade becomes unbound.
5. Compacting drawer: iron ingots in, block/ingot/nugget slots show; take nuggets and blocks;
   `slots` state (open slots) changes; compacting UI; hopper in and out.
6. Drawer puller on a slot, put the detached drawer back; add_detached_upgrade recipe.
7. Key buttons on a controller; keyring sneak-use rotation.
8. Recipes: add_upgrade (drawer + upgrade), keyring, remote_group_upgrade.
9. Framing table: place (two halves), frame a drawer, take the result, break a framed drawer and
   place it back; retrim a drawer with a trim (sneak).
10. Conversion upgrade with an `itemEquivalenceGroups` entry.


## Phase 4: host APIs for the rest of Storage Drawers (2026-09-30, HANDOFF-2)

Host (branch `modded`), each listed in `HOOKS.md`:
- `modded.register-component-stream-codec`: a plugin describes a modded component's
  `networkSynchronized` stream codec once (composite, primitives, item stacks, lists, optionals);
  the item stack serializer writes and reads the mod's bytes with it. Tested in
  `pumpkin-protocol/tests/modded_component_stream_codec.rs` (`ControllerBinding`, `FrameData`).
- `entity.get-item-stack` / `set-item-stack`, and `entity-contact.entity` (the entity itself).
- Item hooks `stacked-on-me`, `stacked-on-other` (pickup clicks in any menu), `destroyed`
  (item entity killed), and the `not-in-containers` flag (bundles; shulker box menus now use
  `ShulkerBoxSlot` and the shulker box menu type, as vanilla).
- `item-stack.get-tags` / `has-tag`.

Plugin: `frame_data` and `controller_binding` codecs, hopper upgrade (`entityInside`) and magnet
upgrade (tick chain with `activeSpeed` / `idleSpeed`), keyring keys added and taken out in the
inventory, keyring keys spilled when it burns, `canStoreInContainers` (filled and detached),
conversion upgrade tag allow and deny lists.

Build: host clippy and tests (`pumpkin`, `pumpkin-protocol`, `pumpkin-inventory`,
`pumpkin-data`) pass; plugin builds; server boots with the plugin (86 drawer blocks).

Hopper upgrade follow-up: the client draws the hole in the drawer top (`BlockDrawers.getShape`
with the upgrade), but the server kept a full cube, so items rested on top. Fixed with
`modded.set-block-collision-shape` (per position, held and saved by the host, used for entity
movement); the plugin sets `AABB_*_HOPPER` on every drawer save. The upgrade doesn't pull from
containers in 26.3.0.1: it only collects item entities (`entityInside`).

Real-client results (2026-09-30), all work: framing table and framed items (drop, pick,
inventory), remote upgrade (bind, reach, unbind when the controller breaks), hopper upgrade,
magnet upgrade, keyring keys added and taken out in the inventory, keyring keys spilled in lava,
drawers and detached drawers kept out of bundles and shulker boxes, conversion upgrade tags.

## Phase 4: Storage Drawers limits 7 to 10 (2026-09-30, HANDOFF-3)

- 7. `recipe-manager.crafting-recipes-for`: the crafting recipes of a result item, shaped with
  width, height and cells, shapeless with ingredients, each ingredient as item ids (tags
  resolved). The plugin's `findLowerTier` now scans shaped 2x2 and 3x3 recipes like the mod,
  instead of starting from the 1x1 craft.
- 8. `world.set-redstone-output-sides`: weak and strong power per side, held and saved by the
  host (`PumpkinSignals` keeps a byte when every side is the same, else a byte array). The
  redstone upgrade gives weak power on every side and strong power to the block below, like
  `BlockDrawers.getDirectSignal`.
- 9. `storage-slot.pool` and `rate`: slots with the same pool share one count, kept by the host
  in the smallest unit (each slot holds `count / rate`); a hopper moving one item of a slot
  moves `rate` units, capped at the pool's capacity, and every slot of the pool sees it.
  Compacting drawers (alone and through a controller, one pool per drawer) use it, so several
  hoppers in one tick move exactly what the pool holds and no insert overflows it.
- 10. `block-entity-load-event` now fires for a mod's block entities (chunk NBT only) once each
  time their chunk loads and becomes active, after the chunk locks are released. The plugin
  runs `onEntityLoad`: controllers schedule their first tick (cache, hopper storage), drawers
  re-push their hopper storage, redstone and collision shape, flag a controller binding for
  validation, and restart their tick (magnet).
- Fix (plugin): after a restart, a controller's saved hopper storage could come from another
  network layout; the first hopper move compared it slot by slot with the new layout and moved
  items between drawers (seen by the user: a compacting drawer's items split into two normal
  drawers). The old storage is now kept only if it still matches the drawers.

Real-client result (2026-10-03): the user reports items 7 to 10, the controller storage fix and
the hopper facing fix (`6078f6e29`) work in game.
