# Status

Pins: Minecraft 26.3 (protocol 777), Fabric Loader 0.19.5, Fabric API 0.161.0+26.3,
Storage Drawers 26.3.0.1+fabric.

| Phase | State | Real-client result |
|:--|:--|:--|
| 0 Recon | done, see `DESIGN.md` | n/a (no code) |
| 1 Dynamic registries | done: overlay for blocks/states/items/tags/BE, entity, menu and component types; namespaced Anvil palettes; unknown blocks kept | not run (needs Phase 3 handshake) |
| 2 Data import | not started | not run |
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

