# NeoForge

Status: ✅ done, 🟡 partial, ❌ missing, ➖ not needed. The pieces are explained in
[README.md](README.md).

Audited against the NeoForge source for Minecraft 26.3 (branch `26.3.x`): the
`net.neoforged.neoforge.network` package and the patches to `ServerConfigurationPacketListenerImpl`
and `ClientConfigurationPacketListenerImpl`. Implemented so far (`NEOHANDSHAKE`, `pumpkin-neoforge`
crate, `[modded] neoforge_handshake`): detection, channel negotiation, registry sync of the
registries mods add to, and the `c:` tasks; not yet tested with a real client. The rest is the work list
for the NeoForge adapter (`../ROADMAP.md`, item 7).

## Handshake (configuration phase)

From the `ServerConfigurationPacketListenerImpl` patch and `NetworkRegistry`:

1. The server starts configuration by sending `minecraft:unregister` (ad-hoc channels to reset),
   `minecraft:register` (its initial channels), an empty `neoforge:register`
   (`ModdedNetworkQueryPayload`) and a vanilla ping with id `0`.
2. A NeoForge client answers `neoforge:register` with its channels per protocol (configuration,
   play), each with version, flow and whether it is optional. A vanilla or Fabric client
   doesn't, and the pong arrives first.
3. On `neoforge:register` the server negotiates (`NetworkComponentNegotiator`): every
   non-optional channel on either side must exist on the other with the same version and a
   compatible flow. On failure it sends `neoforge:modded_network_setup_failed` (channel id to
   reason) and disconnects; on success it sends `neoforge:network` (the agreed setup) and
   `minecraft:register` with the channels it now listens on.
4. On the pong with id `0`, a client that never sent `neoforge:register` is treated as vanilla
   (`initializeOtherConnection`): the server disconnects it if any required channel exists,
   else registers its optional configuration channels.
5. Then vanilla configuration runs, with NeoForge's tasks in front (below).

| Piece | Channel / packet | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Detection | Client's `neoforge:register` reply before the ping `0` pong | `ServerConfigurationPacketListenerImpl` patch | 🟡 `pumpkin/src/net/java/loaders.rs` (one ping for Fabric and NeoForge) | Real-client test |
| Channel negotiation | `neoforge:register`, `neoforge:network`, `neoforge:modded_network_setup_failed` | `NetworkRegistry.initializeNeoForgeConnection`, `NetworkComponentNegotiator` | 🟡 `pumpkin-neoforge/src/{negotiation,handshake}.rs` | The server lists only its registry sync channels; mods' channels from the dump and plugins; `modded_network_setup_failed` (the reasons go in the disconnect message instead) |
| Channel registration | `minecraft:register`, `minecraft:unregister` | `MinecraftRegisterPayload`, `NetworkRegistry.onMinecraftRegister` | ✅ shared with Fabric (`pumpkin-fabric/src/wire/register.rs`) | |
| Common packets | `c:version`, `c:register` (same as Fabric) | `CommonVersionTask`, `CommonRegisterTask` | 🟡 run when the client listens on them | The server's play channel list is empty |
| Split payloads | `neoforge:split` | `GenericPacketSplitter`, `SplitPacketPayload` | ❌ | For payloads above the vanilla limit |

## Configuration tasks

Early (before vanilla's registry and tag sync): `ConfigurationInitialization.configureEarlyTasks`.
Then `RegisterConfigurationTasksEvent`: `configureModdedClient`.

| Task | Channel / packet | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Registry sync | `neoforge:frozen_registry_sync_start` (registry names), one `neoforge:frozen_registry` per registry (`RegistrySnapshot`: id map and aliases), `neoforge:frozen_registry_sync_completed` (both ways) | `SyncRegistries`, `RegistryManager.generateRegistryPackets` | 🟡 registries mods added to, with all entries (as for Fabric) | The client accepts a subset (`ClientPayloadHandler`: it only fails on entries it doesn't know); NeoForge's own registries need a NeoForge dump |
| Config sync | `neoforge:config_file` (file name, raw bytes) for each `SYNCED` mod config | `SyncConfig`, `ConfigSync.syncAllConfigs` | ❌ | Plugins provide the config file bytes |
| Data maps | `neoforge:known_registry_data_maps`, reply `neoforge:known_registry_data_maps_reply`; then `neoforge:registry_data_map_sync` in play | `RegistryDataMapNegotiation`, `RegistryDataMapSyncPayload` | ❌ | Data maps from the mod's datapack in the dump |
| Extensible enums | `neoforge:extensible_enum_data`, reply `neoforge:extensible_enum_ack` | `CheckExtensibleEnums` | ❌ | Enum extensions from the dump (mods that extend networked enums) |
| Feature flags | `neoforge:feature_flags`, reply `neoforge:feature_flags_ack` | `CheckFeatureFlags` | ❌ | Usually empty; send the mod's flags |
| Mod configuration tasks | Any | `RegisterConfigurationTasksEvent` | 🟡 | Same gap as Fabric: plugin tasks that wait for a reply |

Registries NeoForge syncs (`NeoForgeRegistriesSetup.VANILLA_SYNC_REGISTRIES`, plus its own built
with `sync(true)`): `sound_event`, `mob_effect`, `block`, `entity_type`, `item`, `fluid`,
`particle_type`, `block_entity_type`, `menu`, `command_argument_type`, `stat_type`,
`villager_type`, `villager_profession`, `data_component_type`, `recipe_serializer`, `attribute`,
`potion`, `number_format_type`, `custom_stat`, `position_source_type`, `map_decoration_type`,
`consume_effect_type`, `recipe_display`, `slot_display`, `recipe_book_category`, `recipe_type`,
`point_of_interest_type`, `game_event`, `debug_subscription`; `neoforge:entity_data_serializers`,
`neoforge:fluid_type`, `neoforge:holder_set_type`, `neoforge:ingredient_serializer`,
`neoforge:fluid_ingredient_type`, `neoforge:synced_attachment_types`. Unlike Fabric, it sends
them on every modded connection, not only once a mod adds an entry.

## Play payloads

| Piece | Channel | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Menus with extra data | `neoforge:advanced_open_screen` | `AdvancedOpenScreenPayload` (`IMenuTypeExtension.create`) | ❌ | Same shape as Fabric's `open_screen`; reuse `modded.open-menu` |
| Entities with spawn data | `neoforge:advanced_add_entity` | `AdvancedAddEntityPayload` (`IEntityWithComplexSpawn`) | ❌ | With modded entities |
| Container data above a short | `neoforge:advanced_container_set_data` | `AdvancedContainerSetDataPayload` | ❌ | Menus whose data slots exceed 16 bits |
| Light data for extra sections | `neoforge:auxiliary_light_data` | `AuxiliaryLightDataPayload` | ➖ | Only with NeoForge's extended light API |
| Recipe content | `neoforge:recipe_content` (recipe types the client asked for) | `RecipeContentPayload` | ❌ | Like Fabric's recipe sync |
| Attachments | `neoforge:sync_attachments` | `SyncAttachmentsPayload`, `AttachmentSync` | ❌ | With the host-held attachments planned for Fabric |
| Data maps | `neoforge:registry_data_map_sync` | `RegistryDataMapSyncPayload` | ❌ | |
| Config files | `neoforge:config_file` (also in play, on reload) | `ConfigFilePayload` | ❌ | |

## Encoding differences on NeoForge connections

`RegistryFriendlyByteBuf` carries the connection type; some vanilla codecs change for NeoForge
clients:

| Piece | Source | Status | Left |
|:--|:--|:--|:--|
| Custom ingredients | `IngredientCodecs`: marker then `neoforge:ingredient_serializer` id and data, only to NeoForge clients | ❌ | Write custom ingredients for NeoForge clients, vanilla form otherwise |
| Custom holder sets | `ByteBufCodecs` patch: negative counts select a `neoforge:holder_set_type` | ❌ | Only when a mod uses them |
| Non-synced registries | `ByteBufCodecs` patch refuses id syncing for registries without `doesSync()` | ➖ | |
| Vanilla clients | `VanillaConnectionNetworkFilter` drops modded attributes, argument types and tag registries for vanilla clients | ➖ | Pumpkin only sends what the client can read already |

## Other pieces

| Piece | Status | Notes |
|:--|:--|:--|
| Tags | 🟡 | Vanilla packet; the NeoForge registry sync must run first, as in the patch |
| Component codecs | ✅ | Loader-independent (`register-component-stream-codec`) |
| Mod list check | ➖ | Channel negotiation does the checking |
| Data dump | ❌ | The Extractor is a Fabric mod; needs a NeoForge build |
| Real client tested | ❌ | |

## Order of work

In `../ROADMAP.md`, item 7:

1. Detection, negotiation (`neoforge:register` / `network` / `modded_network_setup_failed`) and
   the ping-ordered start of configuration.
2. Registry sync of all `doesSync()` registries, then `c:version` / `c:register`.
3. A NeoForge build of the Extractor, and a test mod.
4. `advanced_open_screen` (menus), then config sync, data maps and extensible enum checks.
5. Recipe content, attachments, `neoforge:split`, custom ingredients for NeoForge clients.
