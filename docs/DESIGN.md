# Pumpkin-Modded: design

Goal: a Fabric client with a content mod installed joins this Pumpkin fork and plays with that
mod's content. We never load the mod's Java code on the server. The mod's server-side logic is
rewritten as a Pumpkin WASM plugin, and Pumpkin learns the mod's registry entries from a data
dump.

This document is the Phase 0 recon. Every claim below was checked against source. Citations use
these pinned revisions:

| Source | Revision |
|:--|:--|
| Pumpkin (fork base) | `4426d1113` |
| fabric-api, branch `26.3` | `ba0d6c0dc` (version 0.161.0) |
| Extractor | `master` of 2026-09-26 |
| PumpkinVoice | `0f9edf338` |
| Storage Drawers | jar `26.3.0.1+fabric` decompiled with Vineflower |

## 1. Version pins

| Component | Version | Where it comes from |
|:--|:--|:--|
| Minecraft | 26.3, protocol 777 | `CURRENT_MC_VERSION` and `LOWEST_SUPPORTED_MC_VERSION` in `crates/pumpkin-data/src/generated/packet.rs:3-6`, protocol in `crates/pumpkin-util/src/version.rs:145` |
| Fabric Loader | 0.19.5 | newest stable for 26.3 on meta.fabricmc.net, and what the Extractor and Storage Drawers require (`fabric.mod.json`: `fabricloader >=0.19.5`) |
| Fabric API | 0.161.0+26.3 | newest 26.3 build on maven.fabricmc.net, same as the `26.3` branch we read |
| Storage Drawers | 26.3.0.1+fabric (Modrinth `wE2d96Vp`) | the only required dependency is Fabric API; needs Java ≥ 25 |
| Java | Temurin 25.0.4.1 (portable, in `tools/`) | needed for the Extractor, decompiling and the test client |

Storage Drawers' GitHub repository has no `26.3` branch (the newest is `26.2`), so the decompiled
release jar is the source of truth for the mod's formats.

Plugin ABI: PR #3713 (`pumpkin:plugin@0.2.0`, async ABI) is still open. It only defines WIT; its
own description says binding generation and runtime wiring come in later PRs. We build on
`pumpkin:plugin@0.1.0` (`crates/pumpkin-plugin-wit/v0.1/`) and keep new WIT additive so it can
move to 0.2 later.

## 2. Pumpkin internals

### 2.1 Registry generation

- The Extractor (a Fabric mod) dumps vanilla data to JSON in `assets/`. `tools/pumpkin-codegen`
  turns that JSON into Rust under `crates/pumpkin-data/src/generated/`, which is committed and
  never edited by hand.
- Blocks: `Block` (`crates/pumpkin-data/src/blocks.rs:19`) is `'static` data. `name` is the path
  only, with no namespace (`"diamond_ore"`). `states` is a `&'static [BlockState]`.
  `item_id: u16` points into the item registry.
- `BlockId(u16)` and `BlockStateId(u16)` are newtypes whose invariant is `id < COUNT`
  (`blocks.rs:328`, `block_state.rs:252`). `COUNT` is the length of a generated static array.
- Lookups are `const fn` over static arrays and use `std::hint::assert_unchecked(id < COUNT)`:
  `BlockState::from_id` (`generated/block.rs:21897`), `Block::from_id` (`:595443`),
  `BlockId::from_state_id` (`:599869`). If an ID is out of range, **the result is undefined
  behaviour, not a panic.**
- Name lookups go through a `phf` map keyed by the bare path (`BLOCK_FROM_NAME_MAP`,
  `generated/block.rs:595431`). `Block::from_name` strips `minecraft:` first.
- Properties: `Block::properties()` is one generated `match` over `BlockId` that returns a boxed
  `BlockProperties` (`generated/block.rs:596605`). The order of state IDs within a block follows
  the vanilla `StateDefinition`.
- Items: `Item { id: u16, registry_key, components }` (`generated/item.rs:22`).
  `Item::from_id` is a `const fn` `match` that returns `Option` (`:93508`), so it is safe for
  unknown IDs. `from_registry_key` strips `minecraft:`.
- Data components: `enum DataComponent` is a `u8` enum (`generated/data_component.rs:5`), so it is
  closed.
- Menus: `enum WindowType` (`generated/screen.rs:3`) is closed.
- Block-entity types: `BLOCK_ENTITY_TYPES: &[&str]` (`generated/block.rs:3505`), and
  `BlockState::block_entity_type: u16` indexes into it.
- Tags: `phf` maps from tag name to `&'static Tag`, and `Tag.1` is a `&'static [u16]` of raw IDs
  (`generated/tag.rs:30607`). `BlockId::has_tag` does a linear `contains` (`blocks.rs:363`).

### 2.2 Block, item and entity dispatch

- `BlockRegistry` (`crates/pumpkin/src/block/registry.rs:499`) maps
  `[u16; BlockId::COUNT]` to indices into `Vec<Arc<dyn BlockBehaviour>>`. The array is sized at
  compile time.
- `ItemRegistry` (`crates/pumpkin/src/item/registry.rs:23`) is an `FxHashMap<u16, Arc<dyn
  ItemBehaviour>>`. It already works with any ID.
- Block entities load through `block_entity_from_nbt` (`crates/pumpkin/src/block/entities/mod.rs:180`),
  a `match` on the NBT `id` string. Unknown IDs return `None`, so their data is dropped.
- Entities: `from_type()` in `crates/pumpkin/src/entity/type.rs`. Storage Drawers adds no
  entities.
- `BlockBehaviour`, `ItemBehaviour` and `Goal` methods are synchronous and run inside the tick
  (`AGENTS.md`).

### 2.3 Configuration phase

Code lives under `crates/pumpkin/src/net/java/`.

1. `handle_login_acknowledged` (`login/login_acknowledged.rs:5`) sends `minecraft:brand`, server
   links, an optional resource pack, then `CFeatureFlags` and `CKnownPacks` (`:97`).
2. When the client replies with `SKnownPacks`, `handle_known_packs` (`login/known_packs.rs:4`)
   sends every synced dynamic registry as `CRegistryData`, then `CUpdateTags`, then
   **`CFinishConfig` right away** (`:137`).
3. `SAcknowledgeFinishConfig` switches to play (`pending.rs:506`).

Two gaps matter for Fabric:
- `handle_plugin_message` (`pending.rs:587`) keeps only `minecraft:brand` and drops every other
  configuration-phase custom payload.
- `SConfigPong` is read and ignored (`pending.rs:538`).

Pumpkin has no concept of configuration "tasks". Finish-config is sent unconditionally.

### 2.4 Custom payloads

- Play, serverbound: `SCustomPayload` (`net/java/mod.rs:1054`) fires the blocking
  `PlayerCustomPayloadEvent`. For `minecraft:register` it also fires `PlayerRegisterChannelEvent`
  and `PlayerChannelEvent` once per channel.
- Play, clientbound: plugins call `player.send-custom-payload(channel, data)`
  (`crates/pumpkin-plugin-wit/v0.1/player.wit:1354`).
- Pumpkin never sends its own `minecraft:register` / `c:register`. PumpkinVoice doesn't either
  (see 2.6).

### 2.5 Plugin runtime

- WASM components (`wasm32-wasip2`), world `pumpkin:plugin@0.1.0`
  (`crates/pumpkin-plugin-wit/v0.1/plugin.wit`). The host lives in
  `crates/pumpkin/src/plugin/loader/wasm/wasm_host/wit/v0_1/`, the guest SDK in
  `crates/pumpkin-plugin-api`.
- Plugin exports: `init-plugin`, `on-load`, `on-unload`, `handle-event`, `handle-command`,
  `handle-task`, `handle-ipc-message`, AI goal hooks and `handle-generate-phase`.
- Things a plugin can register: event handlers, commands, permissions, enchantments and recipes
  (`recipe.wit:79-85`). **Plugins cannot register block, item or block-entity behaviour, and
  cannot add registry entries.** Phase 4 has to add both.
- The WIT is published as its own repo, so changes must stay additive (`AGENTS.md`, Plugin API
  section). `pumpkin-plugin-api`, `-wit` and `-utils` are MIT/Apache-2.0, so server code can't be
  copied into them.

### 2.6 Precedent: PumpkinVoice

PumpkinVoice (`src/lib.rs`) is a Simple Voice Chat backend written as a v0.1
plugin:
- It listens to `PlayerCustomPayloadEvent` and filters on its `voicechat:*` channels
  (`src/handlers/custom_payload.rs`).
- It replies with `send_custom_payload` (`src/net/sync.rs`).
- It encodes packets by hand, one Rust struct per Java codec (`src/net/custom_payloads.rs`).
- It never registers channels. That works because Simple Voice Chat talks only in the play phase
  and adds no registry entries.

We'll use the same pattern for Storage Drawers' two payloads (5.3). The registry and handshake
work has no precedent.

## 3. Fabric protocol

### 3.1 Channel registration

- Legacy `minecraft:register` / `minecraft:unregister`: the payload is channel IDs as ASCII
  separated by `\0`, with no length prefix (`fabric-networking-api-v1/.../RegistrationPayload.java:42`).
- `c:version`: a `VarInt` array of supported common-packet versions. Fabric supports `[1]`
  (`CommonVersionPayload.java:32`, `CommonPacketsImpl.java:35`).
- `c:register`: `VarInt version, String protocol ("play" | "configuration"), Collection<Identifier>`
  as a VarInt count followed by strings (`CommonRegisterPayload.java:46`).
- How Fabric's own server detects a Fabric client (`ServerConfigurationNetworkAddon.java:92`,
  `:123`): at the start of configuration it sends `minecraft:register` with its configuration
  channels, then `ClientboundPing(0xFAB71C)`, and pauses configuration.
  - If the client answers with `minecraft:register`, it's a Fabric client.
  - If `Pong` arrives first, it's vanilla, and configuration continues normally.
- Fabric then queues configuration tasks, in order (`CommonPacketsImpl.java:72`):
  1. send `c:version` and wait for the client's `c:version`;
  2. send `c:register(play, server play channels)` and wait for the client's `c:register`;
  3. other tasks, including registry sync.
- Payloads larger than the vanilla clientbound custom-payload limit are split over
  `fabric:split` (`FabricPacketSplitter.java:41`). Payloads under the limit go out unsplit, and
  the client accepts those. **We will keep the registry-sync payload under that limit and never
  split.**

### 3.2 Registry sync

- The server runs `RegistrySyncManager.configureClient` on `BEFORE_CONFIGURE`
  (`FabricRegistryInit.java:38`). It builds a map
  `registry id → (entry id → raw id)` covering every registry that is both SYNCED and MODDED
  (`RegistrySyncManager.java:160-260`). Each map holds all entries, vanilla ones included.
- SYNCED registries include `block`, `item`, `block_entity_type`, `menu`,
  `data_component_type`, `entity_type`, `fluid`, `sound_event`, `particle_type` and others
  (`FabricRegistryInit.java:58` onwards). `creative_mode_tab` and `recipe_serializer` are not
  synced.
- The server sends `fabric:registry/sync` as a configuration task and waits for the serverbound
  `fabric:registry/sync/complete`, which has an empty body (`SyncCompletePayload.java`).
- If the client can't receive `fabric:registry/sync`, it is disconnected unless every registry
  is OPTIONAL (`RegistrySyncManager.java:81-91`).

`fabric:registry/sync` payload (`RegistrySyncPayload.java:122`, write side):

```
VarInt  regNamespaceGroupCount
repeat:
  String  regNamespace            ("" means "minecraft", :215)
  VarInt  regCount
  repeat:
    String  regPath
    byte    attributes            (bit 0 = OPTIONAL, :194)
    VarInt  idNamespaceGroupCount
    repeat (lastBulkLastRawId carries over between namespace groups, starting at 0):
      String  idNamespace         ("" means "minecraft")
      VarInt  bulkCount
      repeat:
        VarInt  firstRawId - lastBulkLastRawId
        VarInt  bulkSize
        bulkSize × String idPath  (consecutive raw IDs)
```

Client side:
- `checkRemoteRemap` (`ClientRegistrySyncHandler.java:100`) disconnects if any entry the server
  sends is unknown to the client.
- `remap(..., REMOTE)` (`MappedRegistryMixin.java:202`) gives client-only entries IDs after the
  server's highest ID.
- Block states are **not** synced. `StateIdTracker.recalcStateMap` (`StateIdTracker.java:80`,
  registered in `BootstrapMixin.java:50`) rebuilds `Block.BLOCK_STATE_REGISTRY` by walking blocks
  in raw-ID order and appending each block's `getPossibleStates()`. **Pumpkin's state IDs must be
  built the same way: block raw-ID order, then the mod's own state order within each block.**
  The same applies to fluid states.

### 3.3 Extended menus (fabric-menu-api-v1)

Storage Drawers opens its GUIs with `ExtendedMenuType`, and fabric-menu-api-v1 opens those with
a play payload `fabric-menu-api-v1:open_screen` in place of the vanilla open-screen packet
(`fabric-menu-api-v1/.../impl/menu/Networking.java:53`, write at `:117`):

```
Identifier   menu type id
byte         containerId          (the class comment says varInt; the code writes a byte, :119)
Component    title                (ComponentSerialization.STREAM_CODEC)
...          type-specific data   (for Storage Drawers: PositionContent, i.e. a BlockPos)
```

## 4. Places in Pumpkin that assume vanilla-only IDs

Each item says what breaks for a modded ID.

**pumpkin-data**
1. `BlockState::from_id`, `Block::from_id`, `BlockId::from_state_id`: `const fn` over static
   arrays with `assert_unchecked`, so an out-of-range ID is UB. About 180 call sites of
   `BlockState::from_id` and 104 of `Block::from_state_id` outside generated code.
2. `BlockId::new` / `BlockStateId::new` / `new_or_air` compare against the compile-time `COUNT`,
   so modded IDs are rejected or become air.
3. `Block.name` and `Item.registry_key` have no namespace.
   `impl ToResourceLocation for &'static Block` (`blocks.rs:117`) always adds `minecraft:`.
   `from_name` / `from_registry_key` strip `minecraft:`, so `storagedrawers:oak_full_drawers_1`
   can never resolve. About 180 hardcoded `minecraft:` prefix sites in non-generated code.
4. `Block::properties()`: a generated `match`, so a modded block has no properties.
5. `BlockStateId::to_be_network_id` indexes `STATE_ID_TO_BEDROCK` with `assert_unchecked`
   (`generated/block.rs:21910`), so it is UB for modded states. Bedrock needs a fallback mapping.
6. Collision and outline shapes are indices into a global static shape table.
7. Tags: static `phf` plus `&'static [u16]`. Mods add their blocks to vanilla tags such as
   `minecraft:mineable/axe`, which needs a runtime overlay. `CUpdateTags` is built from the same
   static data.
8. `enum DataComponent` (u8) is closed. Item stacks carrying modded components
   (`storagedrawers:frame_data`, `controller_binding`, etc.) can't be decoded, encoded or saved.
9. `enum WindowType` is closed, so there are no modded menu types.
10. `BLOCK_ENTITY_TYPES` is static, and `BlockState.block_entity_type` is a u16 index into it.
11. Loot tables and recipes are generated enums or static tables. Plugin recipe registration
    exists but uses vanilla item IDs.

**pumpkin (server)**

12. `BlockRegistry.block_indices: [u16; BlockId::COUNT]` (`block/registry.rs:500`) would be
    indexed out of bounds.
13. `block_entity_from_nbt` (`block/entities/mod.rs:180`) drops unknown block-entity IDs, so
    modded block-entity data is lost on load.
14. The WASM host's world API builds block lists sized to `BlockId::COUNT`
    (`plugin/loader/wasm/wasm_host/wit/v0_1/world.rs:460`).
15. The configuration phase has no Fabric handshake and no task/wait mechanism (2.3).
16. Command argument parsing (`block_state` / `item` arguments) resolves names through the
    functions in item 3.

**pumpkin-world**

17. Anvil save (`chunk/format/mod.rs:540-548`) already writes palette entries as namespaced names
    with `Properties`. It adds `minecraft:` unless the name already starts with it, which is
    wrong for modded blocks.
18. Anvil load goes through `extract_u16_array` (`chunk/format/mod.rs:106`), which resolves
    compound palette entries by name and turns unknown entries into **air** (`:143`). Loading a
    world without the mod therefore destroys modded blocks silently. Raw numeric palettes
    (Pumpkin's other formats) use `new_or_air`, so numeric IDs aren't stable across mod sets.
19. The network chunk encoder writes global palette state IDs. That is fine once IDs match the
    client, and the Fabric sync guarantees they do.

**Extractor**

20. It writes `blockId.path` and drops the namespace (`Blocks.kt:152`, `Items.kt:40`,
    block entities at `Blocks.kt:283`). With a mod loaded it iterates every entry, so vanilla
    and modded names can collide in the output.

## 5. Proof-of-concept mod: Storage Drawers 26.3.0.1

You chose Storage Drawers. Iron Chests was the first idea, but neither Iron Chests nor Iron
Chests: Restocked has a Fabric 26.3 release. Storage Drawers is bigger than the mission's
"at most one simple block entity", so the work is staged (section 6).

### 5.1 What it registers

Found by reading `core/Mod*.java` in the decompiled jar.

| Registry | Count | Synced by Fabric? |
|:--|:--|:--|
| `block` | about 190 (drawers × wood types × 1x1/1x2/2x2/half, compacting, controller, controller IO, trims, framed, framing table) | yes |
| `item` | block items plus about 60 more (upgrades, keys, keyring, tape, and so on) | yes |
| `block_entity_type` | about 12 | yes |
| `menu` | 6 (`drawer_container_1/2/4`, `comp_2/3`, `framing_table`), all `ExtendedMenuType` | yes |
| `data_component_type` | 5 (`controller_binding`, `drawer_count`, `keyring_content`, `frame_data`, `detached_drawer_content`); only `frame_data` and `controller_binding` sync over the network | yes |
| `recipe_serializer` | custom recipe types | no |
| `creative_mode_tab` | 1 | no, client-side only |

Datapack data in the jar: 133 recipes, 16 block loot tables, block and item tags, and recipe
advancements.

### 5.2 Mod network payloads (play phase)

| Channel | Direction | Format (`network/*.java`) |
|:--|:--|:--|
| `storagedrawers:count_update` | S→C | `int x, int y, int z, int slot, int count` (5 × big-endian int32) |
| `storagedrawers:player_bool_config` | C→S | `String uuid, String key, bool value` |
| `fabric-menu-api-v1:open_screen` | S→C | see 3.3; the extra data is a `BlockPos` |

Drawer contents and labels reach the client through vanilla `ClientboundBlockEntityDataPacket`
NBT, plus `count_update` for count changes.

### 5.3 Why it works as a PoC

- It uses every registry kind the plan covers: blocks with states, items, block entities, menus
  and data components.
- The payload surface is small (two channels), and the formats are plain.
- It has no worldgen and no entities, so Phase 1 can defer entity types.

## 6. Architecture

### 6.1 Crates and flags

The goal is that rebasing onto upstream stays feasible.

- `crates/pumpkin-registry-ext` (new): the generic, loader-agnostic dynamic-registry layer.
  1. It loads mod dumps.
  2. It assigns IDs after vanilla, in dump order, which matches the mod's registration order.
  3. It freezes the result into leaked `'static` tables before the server starts.
  4. It exposes a namespaced lookup plus a raw-ID lookup for every synced registry.

  It knows nothing about Fabric.
- `crates/pumpkin-fabric` (new): the Fabric handshake only. It covers `minecraft:register`,
  `c:version`, `c:register`, `fabric:registry/sync`, sync-complete and `open_screen`. All the
  version-sensitive byte layouts live in one module, `pumpkin-fabric/src/wire/`, one file per
  Fabric payload, each citing its Java source.
- No Cargo feature for the registry overlay (changed during Phase 1). An empty overlay behaves
  exactly like vanilla, and a feature would have doubled every lookup path with `cfg`s. The cost
  with no mods loaded is one predictable branch per lookup. Fabric-specific code still lives in
  its own crate.
- At runtime, with no mods loaded, the server behaves byte-for-byte like upstream. It sends no
  register or ping and runs no extra configuration steps.

### 6.2 How IDs become dynamic (decided: option B)

Everything in section 4 comes back to one question: `'static` + `const fn` lookups versus
registries that are only known at runtime.

**Option A: codegen-time mods.** `pumpkin-codegen` also reads `assets/mods/<modid>/` and emits
modded entries into the generated tables after vanilla.
- Cost: small changes to Pumpkin internals (codegen plus namespaces).
- Performance: none, since it's all still static.
- Downsides: you rebuild the server for every mod set, the mod list is fixed at compile time,
  and it doesn't match Phase 1's "runtime-appended entries".

**Option B: runtime overlay, frozen before start (recommended).** Vanilla stays static and
`const`. Modded entries live in a `OnceLock<ModdedRegistries>` that is filled at startup, then
leaked to `&'static`.
- `from_id` becomes `if id < VANILLA_COUNT { static } else { modded }`. That is one predictable
  branch, and it has to be a non-`const fn`.
- `COUNT`-based checks become runtime counts.
- Tags get a runtime overlay.
- `DataComponent`, `WindowType` and the block-entity type table gain a `Modded(u16)` variant or
  an overlay.
- Cost: this is a **large refactor of pumpkin-data**, roughly 350 call sites, and every `const`
  caller of those lookups has to be checked. It is also the only option that gives
  "drop in a dump and a plugin, restart". Most of the diff sits in `pumpkin-data` and in the
  codegen templates, not in gameplay code, so rebasing stays manageable.

I recommend B, done in this order:
1. Namespaces.
2. Runtime-checked `BlockStateId` / `BlockId` with the vanilla fast path.
3. The block-state and item overlay.
4. Tags.
5. Components, menus and block-entity types.

Per the rules I need your approval before starting it.

### 6.3 Persistence

- Save palettes by full namespaced name (`storagedrawers:oak_full_drawers_1`) with
  `Properties`, which is already Anvil's format once item 17 is fixed.
- Block entities with an unknown ID keep their raw NBT and write it back unchanged. We don't
  drop them.
- Unknown palette names don't become air (approved). They load as `minecraft:bedrock`, which is
  visible and can't be broken in survival. The original palette compound is kept per position in
  `ChunkData::unknown_blocks` and written back on save while the stand-in is still there
  (`pumpkin-world/src/chunk/format/unknown_blocks.rs`). No unknown state ID ever reaches the
  network. This also applies to vanilla worlds that contain unknown names, for example a world
  from a newer Minecraft version. Upstream turned those into air.

### 6.4 Handshake flow in the fork (only when mods are loaded)

```
login_ack → brand, links, [resource pack]
         → minecraft:register(config channels) + Ping(0xFAB71C)
client  → minecraft:register  ⇒ Fabric client  |  Pong first ⇒ vanilla, continue as upstream
         → c:version / wait c:version → c:register(play) / wait c:register
         → FeatureFlags, KnownPacks  (the existing upstream path resumes)
client  → KnownPacks → RegistryData…, UpdateTags (with modded tag entries)
         → fabric:registry/sync / wait fabric:registry/sync/complete
         → FinishConfig
```

- A vanilla client connecting to a modded server is kicked with a message naming the missing
  mods, as Fabric does. Otherwise it would see unknown state IDs.
- A Fabric client **without** the mod gets the same kick, from its own `checkRemoteRemap`.

### Configuration

Everything modded is off unless `pumpkin.toml` turns it on, so a vanilla server behaves as
upstream:

```toml
[modded]
enabled = false          # read mod-data/ and allow the modded plugin API
fabric_handshake = false # Fabric's login handshake and fabric:registry/sync
force_bedrock = false    # keep the Bedrock listener on with modded content
```

With `enabled = true`, the Bedrock listener is turned off (Bedrock clients cannot show modded
content) unless `force_bedrock` is set. With `enabled = false`, no mod data is read, and `register-block-hooks`, `register-item-hooks`
and `modded.open-menu` return an error. Each mod loader's handshake gets its own switch
(`forge_handshake` and `neoforge_handshake` are planned next to `fabric_handshake`).

## 7. Plan and verification

| Phase | Deliverable | Test |
|:--|:--|:--|
| 1 | `pumpkin-registry-ext`, option B refactor, namespaced persistence | vanilla ID snapshot unchanged; chunk with modded blocks round-trips through Anvil |
| 2 | Extractor fork: namespaced, per-mod dump; loader for dump + jar `data/` | dump of Storage Drawers loads; counts match 5.1 |
| 3 | `pumpkin-fabric` handshake + registry sync | `fabric:registry/sync` bytes identical to a real Fabric 26.3 server's capture; client joins, places and breaks drawers |
| 4 | WIT for block/item/BE behaviour, payloads and extended menus; Storage Drawers plugin | 1x1 drawer insert/extract/count sync, then 2x2, compacting, controller |
| 5 | `docs/PORTING_A_MOD.md`, plugin template, version-bump checklist | port dry run |

Tooling: a portable JDK 25 and Vineflower to decompile the mod and Minecraft, the reference
sources at the revisions pinned above, and a real Fabric server and client for packet captures.

## 8. Phase 1 implementation notes

- Generated lookups became vanilla-only (`from_vanilla_id`, `vanilla_properties`, ...). The public
  `Block::from_id`, `BlockState::from_id`, `BlockId::from_state_id`, `Item::from_id`, the name
  lookups and `properties()` are now hand-written in `pumpkin-data`. Each one checks the static
  vanilla table first, then `pumpkin_data::dynamic`. They are no longer `const fn`.
- `BlockId::new` / `BlockStateId::new` check against the runtime total. Generated constants use
  the `const fn from_vanilla`, which fails to compile for a non-vanilla ID.
- `Tag` gained a third field, the tag's own name. That lets `has_tag` consult the overlay: the
  compiler merges identical static slices, so pointer identity can't be used.
- Modded collision/outline shapes are interned against the vanilla shape table and appended past
  its end.
- Chunk direct palettes stay at 16 bits. Vanilla already has 35,723 states, which is more than
  2^15, and modded IDs are capped below `u16::MAX`. The client computes the same
  `ceillog2(total)`.
- Windows: `rustfmt` overflows its 1 MiB main-thread stack on `generated/block.rs`, and codegen
  then silently writes the file unformatted. That happens upstream too, not only with our
  changes. Run codegen with a copy of rustfmt that has a 256 MiB stack, first on `PATH`: copy the
  toolchain's `rustfmt.exe` and run `editbin /STACK:268435456` on the copy (again after each
  toolchain update).
- Known gap for Phase 3: the chunk packet resolves block-entity type IDs by the last path segment
  of the NBT `id` (`net/java/chunk_data/v1_18.rs`), so modded block entities get ID 0.

