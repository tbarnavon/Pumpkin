# Fabric

Status: ✅ done, 🟡 partial, ❌ not started, ➖ not needed for this loader. "To verify" marks
protocol details written from memory, to be checked against the loader's source before
implementing.
The pieces are explained in [README.md](README.md).

| Piece | Status | Where | Notes |
|:--|:--|:--|:--|
| Detection | ✅ | `pumpkin-fabric/src/handshake.rs` | `minecraft:register` back before the ping's pong |
| Channel negotiation | ✅ | `pumpkin-fabric/src/wire/{common,register}.rs` | `c:version`, `c:register`, `minecraft:register` / `unregister` |
| Registry sync | ✅ | `pumpkin-fabric/src/wire/registry_sync.rs`, `sync_map.rs` | `fabric:registry/sync`, `fabric:registry/sync/complete` |
| Tags | ✅ | `pumpkin-protocol/.../update_tags.rs` | Modded and convention (`c:`) tags |
| Component codecs | ✅ | `modded.register-component-stream-codec` | Described by the mod's plugin |
| Configuration payloads | 🟡 | `context.register-configuration-payload` | Sent to every client; no configuration tasks that wait for a reply |
| Login queries | ✅ | `context.register-login-query` | |
| Mod list check | ➖ | | Fabric doesn't check mods on join |
| Config sync | ➖ | | Not part of Fabric API |
| Data dump | ✅ | `extractor/` (Fabric mod), `pumpkin-registry-ext` | Run once per mod |
| Real client tested | ✅ | `STATUS.md` | Storage Drawers 26.3.0.1 |

Fabric API channels found in `refs/fabric-api` but not handled yet (to audit module by module):
`fabric:recipe_sync` and `fabric:recipe_sync/supported_serializers`,
`fabric:custom_ingredient_sync`, `fabric:extended_block_particle_option_sync`, data attachment
sync, extended menu opening data, modded command argument types.

Still to do for Fabric:
- **Synced dynamic registries** (`DynamicRegistries.registerSynced`): mod datapack registries sent
  in configuration. Not checked yet.
- **Recipe sync** (`RecipeSynchronization`): recipes the client needs for its own recipe logic.
- **Attachment sync** (`fabric:attachment_sync`) and **custom synced entity data**
  (`FabricEntityDataRegistry`).
- **Modded particle types and command argument types:** the client must know them when they are
  sent.
- **Configuration tasks:** hold the configuration phase until a mod's reply arrives.
