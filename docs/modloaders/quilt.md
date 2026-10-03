# Quilt

Status: ✅ done, 🟡 partial, ❌ not started, ➖ not needed for this loader. "To verify" marks
protocol details written from memory, to be checked against the loader's source before
implementing.
The pieces are explained in [README.md](README.md).

Quilt Loader runs most Fabric mods, usually with Fabric API through QFAPI. Quilt Standard
Libraries (QSL) have their own registry sync and handshake channels (to verify).

| Piece | Status | Notes |
|:--|:--|:--|
| Fabric mods on a Quilt client | ❌ not tested | May already work through the Fabric handshake |
| QSL registry sync | ❌ | Needed if QSL sync runs instead of Fabric's (to verify) |
| Data dump | ➖ | Same mods as Fabric: the Fabric dump should do |
