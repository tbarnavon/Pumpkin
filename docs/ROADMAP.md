# Roadmap

What to build next, in order. `STATUS.md` records what's done and tested; this file is the
to-do list. Update both when an item lands.

## Goals (2026-10-03)

- **Usable by anybody:** mods are drop-in plugin files, not compiled into the server.
- **No loader-API parity goal:** a mod's server side is rewritten; what must match the loader is
  only what the client sees (handshake, channels, registry sync, payloads, component codecs).
- **Change anything server-side** through a hook catalog planned ahead, so a new mod rarely
  needs fork edits.
- **Never behind upstream:** an LLM reviews upstream merges on a schedule and rebases the fork.

## Performance

Benchmark (release build, 10,000 plugin block entities with a ticker hook, measured with a test
plugin on a fresh world). Before: one guest call per block entity. After: the tick batch, one
guest call per plugin and tick (2026-10-03).

| Case | Tick time before | Per block entity before | Tick time after | Per block entity after |
|:--|:--|:--|:--|:--|
| Empty world | 0.37 ms | - | 0.16 ms | - |
| 10,000 idle vanilla hoppers | 0.37 ms | ~0 | 0.16 ms | ~0 |
| Ticker doing nothing | 395 ms | 39 µs | 4.5 ms | 0.43 µs |
| + 10 `get-block-state-id` | 404 ms | 40 µs | 39.2 ms | 3.9 µs |
| + 1 `get-block-entity-data` | 457 ms | 46 µs | 19.6 ms | 1.9 µs |
| + 1,000-step WASM loop | 365 ms | 36 µs | 5.6 ms | 0.54 µs |

The batch removed the per-call plumbing (`block_on`, store lock, resources, call record): a
ticking block entity now costs ~0.4 µs, so ~100,000 of them fit in a 50 ms tick instead of
~1,250. What remains is the plugin's own work, mostly host calls (~0.35 µs per
`get-block-state-id`, ~1.5 µs per `get-block-entity-data`), which bulk access (item 2) reduces.

1. ~~**Batched per-tick hooks.**~~ done (2026-10-03, `1fa249829`): `ticker`, `entity-inside`,
   `step-on` and `inventory-tick` are queued during the tick and sent as one
   `handle-tick-batch` call per plugin and world. Target (~1 µs per block entity) met: 0.43 µs.
2. ~~**Bulk world access.**~~ done (2026-10-03, `5d96d025f`): `world.get-block-state-ids`,
   `get-block-states-in-box` (up to 32,768 blocks), `set-block-states` and
   `get-block-entity-data-list`. Whole inventories were already one call upstream
   (`inventory.get-all-items` / `set-all-items`).
3. ~~**Worker jobs.**~~ done (2026-10-03, `81f35ff24`): `scheduler.spawn-job(kind, input)` runs
   the plugin's `run-job` export on a worker instance (same `InstancePre`, own store, no
   server) on a dedicated pool of a quarter of the cores; up to 2 idle workers per plugin are
   kept. The output reaches the `handle-job-result` export on a later tick (an export rather
   than an event: no registration, and no change to upstream's event list). In the SDK:
   `scheduler::spawn_job(kind, input, callback)` and `Plugin::run_job`.
4. **Shared-memory threads** in a plugin, once wasmtime supports them for components
   (shared-everything threads proposal).
5. **Parallel plugin ticking by region**, only if a measured bottleneck needs it: mod state
   spans regions.

## Hooks and APIs

6. **Hook catalog.** Research done (2026-10-03): `HOOKS.md`, "Hook catalog", compares Fabric
   API, NeoForge, Paper/Bukkit events and common Mixin targets with what Pumpkin offers, and
   orders the gaps by need ("Gaps by need"). Next: implement them in batches, most needed
   first.
7. **Loader networking.** Each loader's pieces, with channels and source classes, are in
   `modloaders/`. In order:
   - **Fabric (finish):** configuration tasks that wait for a reply; registry sync for the
     other 22 `SYNCED` registries; custom ingredient sync; attachment sync; `fabric:split`;
     recipe sync; synced dynamic and mod-created registries; modded argument types and entity
     data serializers (`modloaders/fabric.md`, "Order of work").
   - **NeoForge adapter**, next to `pumpkin-fabric`: detection, `neoforge:register` negotiation
     and frozen registry sync (`a904c6276`) and NeoForge's synced config (`3035bfca5`) are
     done, and a plain NeoForge client joins and plays (2026-10-03); next a NeoForge build of
     the Extractor and a test mod; `advanced_open_screen`; mods' synced configs, data maps,
     extensible enum and feature flag checks;
     recipe content, attachments, `neoforge:split`, custom ingredients
     (`modloaders/neoforge.md`).
   - **Quilt:** one real-client test of Storage Drawers on Quilt Loader 0.31 (it runs Fabric API
     on 26.3; no QSL exists for 26.x).
   - **Forge:** `\0FORGE` detection and the `forge:handshake` tasks (mod list, channel
     versions, registry sync, config sync) are implemented (`FORGEHS`), waiting for a plain
     Forge client test; then `OpenContainer`, `SpawnEntity` (`modloaders/forge.md`).
8. ~~**Bump `PLUGIN_API_VERSION`**~~ done (2026-10-03): 2 to 3, since `StorageSlot`,
   `PluginSignals` and `set_plugin_redstone_output` changed for native plugins.

## Keeping up with upstream

9. ~~**`FORK_CHANGES.md`**~~ done (2026-10-03): each fork change by area, its commits, files
   and why, plus the files where rebases conflict. Keep it updated with every fork commit.
10. **Headless tests for Storage Drawers:** place a drawer, insert, move items with a hopper,
    restart, check the NBT. Lets an LLM verify a rebase without a real client.
11. **Scheduled upstream sync:** fetch upstream `master`, list PRs merged since the last sync tag,
    classify them against `FORK_CHANGES.md`, rebase `modded` on a branch, run clippy, tests and a
    server boot, and write a report.
12. **Send generic fixes upstream:** forced chunks without load tickets (`c0f42ce91`), hoppers
    loaded from disk facing down (`6078f6e29`).

## Open

- A `/stop` hang reported once, not reproduced.
