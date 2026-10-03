# Mod loader support

What each mod loader's **client** needs from the server, and how far the fork supports it. The
server never runs a loader: a mod's server side is a plugin (see `HOOKS.md`). What has to match
the loader is only what crosses the network, plus a data dump of the mod's registries.

Status: ✅ done, 🟡 partial, ❌ not started, ➖ not needed for this loader.
"To verify" marks protocol details written from memory, to be checked against the loader's source
before implementing.

## Summary

| Loader | Status | Notes |
|:--|:--|:--|
| Fabric | ✅ works | Storage Drawers 26.3.0.1 tested with a real client |
| Quilt | ❌ not tested | Runs Fabric mods; may need its own registry sync (to verify) |
| NeoForge | ❌ not started | Next loader to add (`ROADMAP.md`, item 7) |
| Forge (MinecraftForge) | ❌ not started | After NeoForge; login-phase handshake |
| Others | ➖ | See the end of this file |

## What every loader needs

| Piece | What it is |
|:--|:--|
| Detection | Tell a modded client from a vanilla one, and which loader |
| Channel negotiation | Agree on the custom payload channels both sides know |
| Registry sync | Send the modded registry ids (blocks, items, block-entity types...) so client and server numbers match |
| Tags | Modded tags in the vanilla tag packets |
| Component codecs | Modded item components written the way the mod's client code reads them |
| Configuration and login exchanges | Loader or mod payloads before play (queries, config tasks) |
| Mod list check | Refuse or warn when client and server mods differ |
| Config sync | Server-side mod config sent to the client |
| Data dump | The mod's registries, tags, recipes and loot tables, extracted once from the real mod |

## Fabric

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

Still to do for Fabric:
- **Synced dynamic registries** (`DynamicRegistries.registerSynced`): mod datapack registries sent
  in configuration. Not checked yet.
- **Recipe sync** (`RecipeSynchronization`): recipes the client needs for its own recipe logic.
- **Attachment sync** (`fabric:attachment_sync`) and **custom synced entity data**
  (`FabricEntityDataRegistry`).
- **Modded particle types and command argument types:** the client must know them when they are
  sent.
- **Configuration tasks:** hold the configuration phase until a mod's reply arrives.

## Quilt

Quilt Loader runs most Fabric mods, usually with Fabric API through QFAPI. Quilt Standard
Libraries (QSL) have their own registry sync and handshake channels (to verify).

| Piece | Status | Notes |
|:--|:--|:--|
| Fabric mods on a Quilt client | ❌ not tested | May already work through the Fabric handshake |
| QSL registry sync | ❌ | Needed if QSL sync runs instead of Fabric's (to verify) |
| Data dump | ➖ | Same mods as Fabric: the Fabric dump should do |

## NeoForge

NeoForge negotiates in the configuration phase (to verify):
- a network negotiation where both sides list their payload channels and versions, and the
  client disconnects on a required channel the server lacks;
- registry sync as configuration payloads (a start, one per registry, a completion);
- server config files sent to the client;
- configuration tasks the server must answer.

| Piece | Status | Notes |
|:--|:--|:--|
| Detection | ❌ | |
| Channel negotiation | ❌ | Required channels must be answered or the client disconnects |
| Registry sync | ❌ | |
| Tags | 🟡 | Vanilla packet already carries modded tags; NeoForge tag conventions to check |
| Component codecs | ✅ | Loader-independent (`register-component-stream-codec`) |
| Configuration tasks | ❌ | |
| Config sync | ❌ | Server mod configs sent to the client |
| Data dump | ❌ | The Extractor is a Fabric mod; needs a NeoForge build or a multi-loader one |
| Real client tested | ❌ | |

## Forge (MinecraftForge)

Forge's modern handshake runs in the login phase through login queries (FML handshake: mod list,
channel versions, registry snapshot), to verify for the current version. The fork's
`context.register-login-query` is the starting point.

| Piece | Status | Notes |
|:--|:--|:--|
| Detection | ❌ | |
| FML login handshake | ❌ | Mod list, channels, registries in login queries |
| Tags | 🟡 | As NeoForge |
| Component codecs | ✅ | Loader-independent |
| Config sync | ❌ | |
| Data dump | ❌ | Needs a Forge build of the Extractor |
| Real client tested | ❌ | |

## Others

| Loader | Why not |
|:--|:--|
| Legacy Fabric, Ornithe, Babric | Old Minecraft versions; Pumpkin targets the latest |
| LiteLoader, Rift | Discontinued |
| Paper, Spigot, Bukkit | Server plugin platforms, not client loaders: nothing to sync |
| Bedrock add-ons | Different protocol; Bedrock is turned off while modded content is on |
