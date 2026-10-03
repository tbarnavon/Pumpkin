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
plugin on a fresh world):

| Case | Tick time | Per block entity | Inside the plugin |
|:--|:--|:--|:--|
| Empty world | 0.37 ms | - | - |
| 10,000 idle vanilla hoppers | 0.37 ms | ~0 | - |
| Ticker doing nothing | 395 ms | 39 µs | 0.8 µs |
| + 10 `get-block-state-id` | 404 ms | 40 µs | 6.2 µs |
| + 1 `get-block-entity-data` | 457 ms | 46 µs | 7.3 µs |
| + 1,000-step WASM loop | 365 ms | 36 µs | 0.9 µs |

WASM code and host reads are cheap; ~35 µs per hook call is host plumbing (`block_on`, store
lock, resources, call record). Limit today: ~1,250 ticking plugin block entities per 50 ms tick.

1. **Batched ticker hook.** Implemented (2026-10-03): `ticker`, `entity-inside`, `step-on`
   and `inventory-tick` are queued during the tick and sent as one `handle-tick-batch` call per
   plugin and world, so the async setup is paid once per plugin per tick. Benchmark rerun
   pending; target ~1 µs per block entity.
2. **Bulk world access.** Read or change a whole inventory (and other bulk data) in one call,
   not slot by slot.
3. **Worker jobs.** `scheduler.spawn-job(handler, bytes)` runs the plugin's `run-job` export in
   a worker instance (same `InstancePre`, own memory) on a thread pool; the result comes back as
   an event on a later tick. Workers get no world access. For autocrafting (AE2 calculates on a
   background thread too), pathfinding, large recipe searches.
4. **Shared-memory threads** in a plugin, once wasmtime supports them for components
   (shared-everything threads proposal).
5. **Parallel plugin ticking by region**, only if a measured bottleneck needs it: mod state
   spans regions.

## Hooks and APIs

6. **Hook catalog.** One pass over what mods hook into: Fabric API events, NeoForge events,
   common Mixin targets, Paper events, and what Pumpkin has. Mark existing hooks and gaps, then
   implement the gaps in batches, so an LLM can write a server-side mod against the WIT alone.
   Added hooks and known gaps are tracked in `HOOKS.md`.
7. **Loader networking.** Each loader's pieces, with channels and source classes, are in
   `modloaders/`. In order:
   - **Fabric (finish):** configuration tasks that wait for a reply; registry sync for the
     other 22 `SYNCED` registries; custom ingredient sync; attachment sync; `fabric:split`;
     recipe sync; synced dynamic and mod-created registries; modded argument types and entity
     data serializers (`modloaders/fabric.md`, "Order of work").
   - **NeoForge adapter**, next to `pumpkin-fabric`: detection and `neoforge:register`
     negotiation; frozen registry sync; a NeoForge build of the Extractor and a test mod;
     `advanced_open_screen`; config sync, data maps, extensible enum and feature flag checks;
     recipe content, attachments, `neoforge:split`, custom ingredients
     (`modloaders/neoforge.md`).
   - **Quilt:** one real-client test of a Fabric mod; no QSL exists for 26.x.
   - **Forge:** `\0FORGE` detection, then the `forge:handshake` tasks (mod list, channel
     versions, registry sync, config sync), `OpenContainer`, `SpawnEntity`
     (`modloaders/forge.md`).
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
