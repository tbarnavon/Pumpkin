# Status

Pins: Minecraft 26.3 (protocol 777), Fabric Loader 0.19.5, Fabric API 0.161.0+26.3,
Storage Drawers 26.3.0.1+fabric.

| Phase | State | Real-client result |
|:--|:--|:--|
| 0 Recon | done, see `DESIGN.md` | n/a (no code) |
| 1 Dynamic registries | done: overlay for blocks/states/items/tags/BE, entity, menu and component types; namespaced Anvil palettes; unknown blocks kept | not run (needs Phase 3 handshake) |
| 2 Data import | done: Extractor mod dump (`extractor/`, branch `modded-dump`) + `pumpkin-registry-ext` loader; recipes and loot tables merged into datapacks | server boots with Storage Drawers data; client join not expected to work before Phase 3 |
| 3 Fabric handshake | not started | not run |
| 4 Plugin API + Storage Drawers plugin | not started | not run |
| 5 Porting kit | not started | not run |

## Phase 1 tests

- `crates/pumpkin-data/tests/dynamic_registries.rs`: every vanilla state keeps its block and ID
  after mods are installed. Modded blocks, states, items and block-entity types are numbered
  directly after vanilla. Properties, shapes and tags (including additions to vanilla tags)
  resolve. A failed freeze installs nothing.
- `crates/pumpkin-world/tests/modded_chunk_roundtrip.rs`: an Anvil chunk with a modded block
  round-trips by namespaced name and properties. A block from a missing mod loads as the
  bedrock stand-in and is written back unchanged, and it is dropped once the stand-in is replaced.
- Existing suites still pass: `cargo test -p pumpkin-data -p pumpkin-world -p pumpkin`. Clippy is
  clean on the touched crates.
- Not yet verified with a real client: that needs the Phase 2 dump loader and the Phase 3
  handshake.

## Phase 2

- Dump: `source tools/gradle-env.sh && cd extractor && ./gradlew runModDump -PmodDumpNamespaces=storagedrawers`.
  It writes `run-moddump/pumpkin_extractor_output/mods/<ns>.json` and `fabric_registry_sync.json`,
  the exact map a real Fabric server sends.
- Install: copy `mods/<ns>.json` into `<server>/mod-data/`. On startup Pumpkin checks that every
  ID matches the dump and refuses to start on a mismatch.
- Storage Drawers 26.3.0.1: 150 blocks, 965 states, 166 items, 9 block-entity types, 6 menus,
  5 component types. The IDs match the reference server; covered by
  `crates/pumpkin-registry-ext/tests/storage_drawers_dump.rs`.
- Boot test (2026-09-29): 127 of 132 recipes imported. 5 were skipped because they use Storage
  Drawers' custom recipe serializers (Phase 4). All 15 loot tables were imported.
- Block drops now look up `<namespace>:blocks/<path>`, as vanilla `Block.getLootTable` does.

