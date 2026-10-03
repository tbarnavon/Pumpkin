# Mod loader support

What each mod loader's **client** needs from the server, and how far the fork supports it. The
server never runs a loader: a mod's server side is a plugin (see `../HOOKS.md`). What has to
match the loader is only what crosses the network, plus a data dump of the mod's registries.

One file per loader, each listing every client-facing network piece with its channel ids, the
loader's source class, the fork's status and what's left.

Status: ✅ done, 🟡 partial, ❌ missing, ➖ not needed. Each file was audited against the
loader's source for Minecraft 26.3 and cites the classes.

## Summary

| Loader | Status | File |
|:--|:--|:--|
| Fabric | 🟡 what Storage Drawers 26.3.0.1 needs, tested with a real client; the rest is ordered in the file | [fabric.md](fabric.md) |
| Quilt | 🟡 expected to work through the Fabric handshake (Quilt Loader 0.31 runs Fabric API on 26.3; no QSL for 26.x); not tested | [quilt.md](quilt.md) |
| NeoForge | 🟡 handshake, registry sync and NeoForge's config done; a plain NeoForge client joins (2026-10-03); no mods tested yet | [neoforge.md](neoforge.md) |
| Forge (MinecraftForge) | 🟡 `forge:handshake` tasks done; a plain Forge client joins (2026-10-04); no mods tested yet | [forge.md](forge.md) |

Others considered: Sinytra Connector runs Fabric mods on NeoForge, so its clients use the
NeoForge handshake with Fabric API's channels on top; it needs nothing beyond those two files.

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

## Out of scope

| Loader | Why not |
|:--|:--|
| Legacy Fabric, Ornithe, Babric | Old Minecraft versions; Pumpkin targets the latest |
| LiteLoader, Rift | Discontinued |
| Paper, Spigot, Bukkit | Server plugin platforms, not client loaders: nothing to sync |
| Bedrock add-ons | Different protocol; Bedrock is turned off while modded content is on |
