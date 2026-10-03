# NeoForge

Status: ✅ done, 🟡 partial, ❌ not started, ➖ not needed for this loader. "To verify" marks
protocol details written from memory, to be checked against the loader's source before
implementing.
The pieces are explained in [README.md](README.md).

NeoForge negotiates in the configuration phase (to verify):
- a network negotiation where both sides list their payload channels and versions, and the
  client disconnects on a required channel the server lacks;
- registry sync as configuration payloads (a start, one per registry, a completion);
- server config files sent to the client;
- configuration tasks the server must answer.

| Piece | Status | Notes |
|:--|:--|:--|
| Detection | ❌ | |
| Channel negotiation | ❌ | Required channels must be answered or the client disconnects |
| Registry sync | ❌ | |
| Tags | 🟡 | Vanilla packet already carries modded tags; NeoForge tag conventions to check |
| Component codecs | ✅ | Loader-independent (`register-component-stream-codec`) |
| Configuration tasks | ❌ | |
| Config sync | ❌ | Server mod configs sent to the client |
| Data dump | ❌ | The Extractor is a Fabric mod; needs a NeoForge build or a multi-loader one |
| Real client tested | ❌ | |
