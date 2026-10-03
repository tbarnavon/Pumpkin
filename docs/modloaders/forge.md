# Forge (MinecraftForge)

Status: ✅ done, 🟡 partial, ❌ not started, ➖ not needed for this loader. "To verify" marks
protocol details written from memory, to be checked against the loader's source before
implementing.
The pieces are explained in [README.md](README.md).

Forge's modern handshake runs in the login phase through login queries (FML handshake: mod list,
channel versions, registry snapshot), to verify for the current version. The fork's
`context.register-login-query` is the starting point.

| Piece | Status | Notes |
|:--|:--|:--|
| Detection | ❌ | |
| FML login handshake | ❌ | Mod list, channels, registries in login queries |
| Tags | 🟡 | As NeoForge |
| Component codecs | ✅ | Loader-independent |
| Config sync | ❌ | |
| Data dump | ❌ | Needs a Forge build of the Extractor |
| Real client tested | ❌ | |
