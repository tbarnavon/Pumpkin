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

Benchmark (release build, 10,000 plugin block entities with a ticker hook, `bench-plugin/` on
`run-bench/`):

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

1. **Batched ticker hook.** One guest call per plugin per tick with every ticking block entity,
   and a call path without per-call async setup. Rerun the benchmark; target ~1 µs per block
   entity.
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
7. **NeoForge networking adapter**, next to `pumpkin-fabric`. Per-loader status and what's
   left (Fabric, Quilt, NeoForge, Forge) is in `MODLOADERS.md`.
8. **Bump `PLUGIN_API_VERSION`** (`plugin/mod.rs`): `StorageSlot`, `PluginSignals` and
   `set_plugin_redstone_output` changed, so old native plugins would pass the check and break.

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

- Real-client tests of HANDOFF-3 items 7 to 10 and the hopper fix, then record them in
  `STATUS.md`.
- Host bug 12: a `/stop` hang, not reproduced.
