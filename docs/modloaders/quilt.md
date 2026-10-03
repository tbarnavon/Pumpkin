# Quilt

Status: ✅ done, 🟡 partial, ❌ missing, ➖ not needed. The pieces are explained in
[README.md](README.md).

Checked on 2026-10-03 against the Quilt Loader `0.31.0-beta.4` source and Quilt's release
listings:

- **Quilt Loader runs on 26.3.** Quilt's version metadata lists 26.3 with loader builds, and
  `McVersionLookup` parses the 26.x version scheme. Its `quilt.mod.json` `provides`
  `fabricloader` 0.19.5, the same Fabric Loader version this fork targets, so Fabric mods and
  Fabric API load on it unchanged.
- **The Quilt libraries stopped before 26.x.** The newest Quilt Standard Libraries (QSL) release
  is for 1.20.1 (branches go to 1.21.5), and Quilted Fabric API's newest is an alpha for 1.21.
  A 26.3 Quilt client therefore runs the regular Fabric API, and its network traffic is
  Fabric's.
- **The loader adds nothing to the protocol** except the client brand: `Hooks.insertBranding`
  sends `quilt` (or `<brand>,quilt`) where Fabric sends `fabric`. The fork detects Fabric
  clients by `minecraft:register` before the ping's pong, not by brand, so this changes
  nothing.

| Piece | Status | Notes |
|:--|:--|:--|
| Fabric mods on a Quilt client | 🟡 expected to work, not tested | Same handshake as Fabric (see [fabric.md](fabric.md)); one real-client test with Storage Drawers needed |
| Client brand `quilt` | ✅ | Only logged by the server |
| QSL registry sync and channels | ➖ | No QSL for 26.x |
| Data dump | ➖ | Same mods as Fabric: the Fabric dump |
