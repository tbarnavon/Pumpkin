# Quilt

Status: ✅ done, 🟡 partial, ❌ missing, ➖ not needed. The pieces are explained in
[README.md](README.md).

Quilt Loader runs Fabric mods. The Quilt Standard Libraries (QSL), which had their own registry
sync, have no Minecraft 26.x branch: the newest is `1.21` (checked 2026-10-03). On 26.3 a Quilt
client runs Fabric API, so it speaks Fabric's protocol and needs nothing beyond
[fabric.md](fabric.md).

| Piece | Status | Notes |
|:--|:--|:--|
| Fabric mods on a Quilt client | ❌ not tested | Should work through the Fabric handshake; one real-client test needed |
| QSL registry sync | ➖ | No QSL for 26.x |
| Data dump | ➖ | Same mods as Fabric: the Fabric dump |
