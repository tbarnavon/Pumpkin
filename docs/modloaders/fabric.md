# Fabric

Status: ✅ done, 🟡 partial, ❌ missing, ➖ not needed. The pieces are explained in
[README.md](README.md).

Audited against the Fabric API 0.161.0+26.3 source, module by module: every module that
registers a payload type (`PayloadTypeRegistry`), adds a configuration task, marks a registry
`SYNCED`, or mixes into a packet or packet listener. Modules not listed (rendering, models,
key mappings, screens, creative tabs, item, block, transfer, loot, biome, dimensions, events,
lifecycle, permissions, resource conditions, data generation, game tests, serialization,
sound, message, debug) have no client-facing network part of their own: their data either
stays on the server or reaches the client through vanilla packets the fork already sends.

Fabric's support today is the minimum Storage Drawers 26.3.0.1 needs, tested with a real
client (`STATUS.md`).


## On 1.21.1

This branch serves Fabric API 0.116.17+1.21.1 clients. The tables below describe 26.3; the
handshake (`minecraft:register`, ping, `c:version`, `c:register`) is the same on 1.21.1, but
these pieces differ:

| Piece | 1.21.1 | Source (1.21.1) |
|:--|:--|:--|
| Registry sync | `fabric:registry/sync/direct`: the same body without the attributes byte, cut into payloads of at most 1 MiB and ended by an empty one | `DirectRegistryPacketHandler` |
| Synced registries | Every `SYNCED` registry a mod adds to; Storage Drawers adds `recipe_serializer`, which is synced too. Data component ids count only 1.21.1's 57 vanilla components | `RegistrySyncManager.createAndPopulateRegistryMap` |
| Optional registries | No flag on the wire | `DirectRegistryPacketHandler` |
| Extended menus | `fabric-screen-handler-api-v1:open_screen`, same layout | `impl/screenhandler/Networking` |

Fabric's own payload is its buffer's whole backing array, padded with zeros; Pumpkin sends the
exact bytes, which the client reads the same way.

## fabric-networking-api-v1

The base every other module builds on. Source: `impl/networking/*`, `mixin/networking/*`.

| Piece | Channel / packet | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Detection | `minecraft:register` from the client, before the server's ping `0xFAB71C` is answered | `ServerConfigurationNetworkAddon.onClientReady` | ✅ `pumpkin-fabric/src/handshake.rs` | |
| Channel registration | `minecraft:register`, `minecraft:unregister` (NUL-separated ids), all phases | `RegistrationPayload`, `AbstractChanneledNetworkAddon` | ✅ configuration; 🟡 play | Send `minecraft:register` for channels a plugin starts listening to after join |
| Common packets version | `c:version` (int array), configuration and play | `CommonPacketsImpl`, `CommonVersionPayload` | ✅ | |
| Common channel list | `c:register` (version, phase `play`, channel list), configuration and play | `CommonPacketsImpl`, `CommonRegisterPayload` | ✅ | |
| Configuration tasks | Tasks run in order; each holds the phase until `completeTask` | `ServerConfigurationPacketListenerImplMixin` | 🟡 | `context.register-configuration-payload` sends and moves on; a plugin can't hold the phase for its reply (needed by the modules below that use tasks) |
| Early task execution | Tasks may run before vanilla's own, after the ping | `ServerConfigurationPacketListenerImplMixin` (`earlyTaskExecution`) | ✅ | The handshake already orders Fabric's tasks first |
| Login queries | Any channel, `ClientboundCustomQueryPacket` | `ServerLoginNetworkAddon` | ✅ `context.register-login-query` | |
| Large payloads | `fabric:split`: a payload above ~1 MiB sent in chunks, first one prefixed with the total size | `FabricPacketSplitter`, `FabricSplitPacketPayload`, `ClientboundCustomPayloadPacketMixin` | ❌ | Needed once registry sync, recipe sync or attachment sync of a large pack exceeds the vanilla limit (1,048,576 bytes); the client also splits serverbound payloads registered as large |
| Play payloads from plugins | Any channel | `ServerPlayNetworking` | ✅ upstream `player.send-custom-payload` | |

## fabric-registry-sync-v0

Source: `impl/registry/sync/*`, `FabricRegistryInit`, `RegistrySyncManager`.

| Piece | Channel / packet | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Registry sync | `fabric:registry/sync` (configuration, large), answered by `fabric:registry/sync/complete` | `RegistrySyncPayload`, `SyncCompletePayload`, `RegistrySyncManager.configureClient` | 🟡 `pumpkin-fabric/src/wire/registry_sync.rs`, `sync_map.rs` | Only 6 of the 28 `SYNCED` registries are covered (below) |
| Incompatible clients | Disconnect with a message when the client can't receive `fabric:registry/sync` and a registry isn't optional | `RegistrySyncManager.configureClient` | ✅ | |
| Optional registries | `RegistryAttribute.OPTIONAL`: the client may lack the registry | `RegistrySyncPayload` | ✅ flag written (always false) | Set it when the mod's dump marks a registry optional |
| Synced dynamic registries | `DynamicRegistries.registerSynced`: mod datapack registries in vanilla's `ClientboundRegistryDataPacket` | `DynamicRegistriesImpl`, `RegistrySynchronizationMixin` (`SKIP_WHEN_EMPTY`) | ❌ | Send mod dynamic registries (and their entries) from the dump in registry data; skip empty ones flagged `SKIP_WHEN_EMPTY` |
| Mod-created registries | `FabricRegistryBuilder...attribute(SYNCED)` | `FabricRegistryBuilder` | ❌ | Dump and sync any modded registry marked `SYNCED` |
| Known packs | Only packs enabled at server start are offered | resource-loader `ServerConfigurationPacketListenerImplMixin` | ➖ | Pumpkin offers only `minecraft:core` |

`SYNCED` registries in `FabricRegistryInit` (sent when a mod added to them):

| Registry | Fork |
|:--|:--|
| `item`, `block`, `block_entity_type`, `data_component_type`, `menu`, `entity_type` | ✅ |
| `sound_event`, `fluid`, `mob_effect`, `potion`, `particle_type`, `attribute`, `game_event`, `villager_type`, `villager_profession`, `point_of_interest_type`, `stat_type`, `custom_stat`, `command_argument_type`, `number_format_type`, `position_source_type`, `data_component_predicate_type`, `map_decoration_type`, `consume_effect_type`, `recipe_display`, `slot_display`, `recipe_book_category`, `debug_subscription` | ❌ Not in the overlay yet; each needs its runtime overlay first (`FORK_CHANGES.md`, area 1) |
| `fabric-object-builder-api-v1:tracked_data_handler` (Fabric's own) | ❌ See object builder |

## fabric-recipe-api-v1

| Piece | Channel / packet | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Custom ingredients | `fabric:custom_ingredient_sync`: configuration task; server sends protocol 1, client answers with the ingredient serializers it knows | `CustomIngredientSync` | 🟡 recipes with `fabric:all_of` and others load (`2717a2ac3`) | Run the task, and write custom ingredients in recipe-book packets only for clients that support them (vanilla fallback otherwise) |
| Recipe sync | `fabric:recipe_sync/supported_serializers` (serverbound, configuration), then `fabric:recipe_sync` (clientbound, play, large) with full recipes of the serializers the client asked for | `RecipeSyncImpl`, `ClientboundRecipeSyncPayload` | ❌ | Recipes from the dump re-encoded with each serializer's stream codec; sent on join and datapack reload |

## fabric-data-attachment-api-v1

| Piece | Channel / packet | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Accepted attachments | `fabric:accepted_attachments_v1`: configuration task; server asks, client lists the attachment types it can receive | `AttachmentSync`, `ClientboundRequestAcceptedAttachmentsPayload`, `ServerboundAcceptedAttachmentsPayload` | ❌ | |
| Attachment sync | `fabric:attachment_sync_v1` (play, large, bundled when several): target (global, world, entity, block entity, chunk), type id, value | `ClientboundAttachmentSyncPayload`, `AttachmentChange` | ❌ | A host-held attachment store with sync rules, filled by plugins (data first) |

## fabric-menu-api-v1

| Piece | Channel / packet | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Extended menus | `fabric-menu-api-v1:open_screen`: menu type, title, extra data | `impl/menu/Networking` | ✅ `modded.open-menu` | |

## fabric-particles-v1

| Piece | Channel / packet | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Block particle position | `fabric:extended_block_particle_option_sync` (empty, configuration): tells the server the client reads a `BlockPos` after block particles (marker `-1`) | `ExtendedBlockParticleOptionSync`, `ExtendedBlockParticleOptionStreamCodec` | ➖ | The client reads vanilla's encoding when no marker is sent |
| Modded particle types | Raw ids in `ClientboundLevelParticlesPacket` | `FabricParticleTypes` | ❌ | `particle_type` registry sync and a way for plugins to send them |

## fabric-command-api-v2

| Piece | Channel / packet | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Modded argument types | Raw ids in `ClientboundCommandsPacket`, after `command_argument_type` registry sync | `ArgumentTypeRegistry` | ❌ | Sync the registry and let plugin commands use a mod's argument type (id + properties bytes) |
| Entity selector options | Parsed on the server only | `EntitySelectorOptionRegistry` | ➖ | |

## fabric-object-builder-api-v1

| Piece | Channel / packet | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Custom entity data | Entity data serializer ids after vanilla's, from the synced registry `fabric-object-builder-api-v1:tracked_data_handler` | `FabricEntityDataRegistryImpl` | ❌ | Needed with modded entities: sync the registry and write the serializer ids in entity metadata |
| Villager trades, block entity types | Server side | `TradeOfferHelper`, `FabricBlockEntityTypeBuilder` | ➖ | Block-entity types are synced through registry sync |

## Other pieces

| Piece | Status | Notes |
|:--|:--|:--|
| Tags | ✅ `pumpkin-protocol/.../update_tags.rs` | Modded and convention (`c:`) tags; `fabric-tag-api-v1` aliases stay on the server |
| Component codecs | ✅ `modded.register-component-stream-codec` | Described by the mod's plugin |
| Mod list check | ➖ | Fabric doesn't compare mod lists on join |
| Config sync | ➖ | Not part of Fabric API (mods use their own channels) |
| Data dump | ✅ the Extractor (a Fabric mod), `pumpkin-registry-ext` | Run once per mod |

## Order of work

In `../ROADMAP.md`, item 7a. Most needed first:

1. Configuration tasks that wait for the client's reply (blocks 3, 4, 6).
2. Registry sync for the remaining `SYNCED` registries, starting with `sound_event`,
   `particle_type`, `mob_effect`, `potion` and `fluid` (common in content mods).
3. Custom ingredient sync task, with vanilla fallback encoding.
4. Attachment sync (host-held attachments).
5. `fabric:split` for large clientbound payloads.
6. Recipe sync.
7. Synced dynamic registries and mod-created registries from the dump.
8. Modded command argument types; custom entity data (with modded entities).
