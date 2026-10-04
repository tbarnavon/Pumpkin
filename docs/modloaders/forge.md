# Forge (MinecraftForge)

Status: ✅ done, 🟡 partial, ❌ missing, ➖ not needed. The pieces are explained in
[README.md](README.md).

Audited against the MinecraftForge source for Minecraft 26.3 (branch `26.3`): the
`net.minecraftforge.network` package. The configuration tasks are implemented (`b708340f8`,
`pumpkin-forge` crate, `[modded] forge_handshake`). Tested 2026-10-04: a plain Forge 26.3
client joins (mod list, channel versions and an empty registry sync exchanged).

Unlike NeoForge, the modern Forge handshake runs in the **configuration** phase on one
channel, `forge:handshake` (a `SimpleChannel`: a VarInt discriminator, then the message), not in
login queries. The login channel `forge:login` only carries `LoginWrapper` from the client.

## Handshake

1. **Detection:** the client appends `\0FORGE` (optionally followed by a network version, `0`
   today) to the server address in the vanilla handshake packet (`NetworkContext`).
2. For a modded connection, `ForgeNetworkConfigurationHandler.gatherInit` adds these
   configuration tasks, in order:

| Task | Channel / message | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Detection | `\0FORGE` in the handshake's server address | `NetworkContext.MARKER` | ✅ `pumpkin-forge` `wire::forge_marker`, read in `login_acknowledged.rs` | |
| Vanilla channel list | `minecraft:register` with every known channel | `RegisterChannelsTask`, `ChannelListManager` | 🟡 `forge:handshake`, `forge:login` | Mods' channels |
| Mod list | `forge:handshake` `ModVersions` (mod id to name and version), both ways | `ModVersionsTask`, `ForgePacketHandler.handleModVersions` | 🟡 installed mod ids, version `0` | Real names and versions from the dump |
| Channel versions | `forge:handshake` `ChannelVersions` (channel to version), both ways; `MismatchData` and a disconnect when they don't match | `ChannelVersionsTask`, `NetworkRegistry.validateChannels` | 🟡 Forge's three channels at version 0, plus mods' channels from `modded.register-loader-channels`; another version on the client disconnects | `MismatchData` (the reasons go in the disconnect message instead) |
| Registry sync | `forge:handshake` `RegistryList` (token), then one `RegistryData` (token, registry name, snapshot) per registry, each answered by `Acknowledge` (token) | `SyncRegistriesTask`, `RegistryManager.takeSnapshot(false)` | 🟡 registries mods added to, with all entries (as for Fabric and NeoForge) | Forge's own registries need a Forge dump |
| Config sync | `forge:handshake` `ConfigData` (file name, raw bytes) for each `SERVER` config | `SyncConfigTask` | 🟡 Forge's own `forge-server.toml` with its defaults | Mods' server configs from plugins |

## Play

| Piece | Channel / message | Source | Status | Left |
|:--|:--|:--|:--|:--|
| Entities with spawn data | `forge:handshake` `SpawnEntity` | `network/packets/SpawnEntity` | ❌ | With modded entities |
| Menus with extra data | `forge:handshake` `OpenContainer` (message 8) | `network/packets/OpenContainer`, `IForgeServerPlayer.openMenu` | ✅ `modded.open-menu` sends it to Forge clients | |
| Channel registration changes | `forge:channel_registration` | `ChannelListManager` | ❌ | |
| Split payloads | `forge:split` (parts of 1 MiB) | `filters/VanillaPacketSplitter` | ❌ | Check whether it is still wired in this version |
| Server list | `forgeData` (mods, channels) in the status JSON | `ServerStatusPing` | ➖ | Only changes the client's server list icon |

## Other pieces

| Piece | Status | Notes |
|:--|:--|:--|
| Tags | 🟡 | Vanilla packet; Forge's registry sync runs first |
| Component codecs | ✅ | Loader-independent |
| Data dump | ✅ | The Extractor's Forge build (2026-10-04): the mod's entries, Forge's synced registry snapshots (ids, aliases, overrides, blocked ids) with each snapshot's bytes, and the mod list `ModVersions` needs |
| Real client tested | ✅ | Plain Forge 26.3 joins; with Storage Drawers 26.3.0.1 (Forge build), drawers, their GUI and the framing table work, alongside a Fabric player on the same server (2026-10-04) |

## Order of work

After NeoForge: detection, then the five configuration tasks in the order above (they share
`forge:handshake`), then `OpenContainer`, `SpawnEntity` and `forge:split`.
