# ADR 0013 — MCP SDK, pinned version and how the contract maps onto MCP

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-26 23:15 IST · last edited 2026-11-26 23:15 IST

- **Status:** Accepted (maintainer, 2026-11-26: "yes to Q-A through Q-G" of [MCP_SDK_PROPOSAL.md](../MCP_SDK_PROPOSAL.md),
  with two implementation gates added)
- **Date:** 2026-11-26
- **Decision ID:** D-13
- **Relation to earlier decisions:** implements API_DESIGN §10 (D-19: tool surface, read-only, stdio) and the offline
  policy (stdio only, no network listener). Changes neither.

## Context

D-13 was registered as *"official `rmcp` crate, stdio only; pin version at Phase 3 start"*. The MCP specification
revision 2026-07-28 removed the `initialize` handshake in favour of per-request versioning and `server/discover`, so a
server that speaks only the new revision fails with clients on 2025-11-25 or earlier. The proposal checked the
specification and the SDK's manifests and releases on 2026-11-26. Sources the maintainer confirmed:
- release `rmcp-v3.12.0`: https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.12.0
- manifest at that tag: https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.12.0/crates/rmcp/Cargo.toml
- versioning: https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning
- tools: https://modelcontextprotocol.io/specification/2026-07-28/server/tools and
  https://modelcontextprotocol.io/specification/2025-11-25/server/tools

## Decision

1. **SDK and pin (Q-A, Q-B).** `rmcp = { version = "=3.12.0", default-features = false, features = ["server",
   "transport-io"] }`. No `macros` (tools are generated at run time from the registry), no HTTP or network features.
   Upgrades go through Dependabot and must pass CI and the gates below.
2. **Both protocol eras (Q-C).** The server serves 2026-07-28 clients (per-request version, `server/discover`) and
   2025-11-25 clients (`initialize` handshake), relying on rmcp's dual-era support.
3. **Tool names (Q-D).** Each calculation's tool name is its calculation ID, verbatim. Generic tools `civkit_list`,
   `civkit_describe`, `civkit_call`; `--tools per-calc|generic|both`, default `both` (API_DESIGN §10).
4. **Results (Q-E).** Success: `structuredContent` is the response exactly as `Registry::call_json` returns it, and the
   same JSON is also returned as a text block. A contract error is a result with `isError: true` and the CivKit error
   envelope (`code`, `input`, `index`, `message`) as `structuredContent` and as text. `outputSchema` is an object schema
   with `oneOf` [response, error envelope]. Unknown tools and malformed requests are JSON-RPC protocol errors.
5. **Annotations (Q-F).** Every tool: `readOnlyHint: true`, `openWorldHint: false`.
6. **Crate (Q-G).** `civkit-mcp`, a binary in `integrations/mcp`, a workspace member outside `crates/`. Stdout carries
   only MCP messages; logging goes to stderr. The dependency check is extended so that its tree contains no HTTP crate
   (`reqwest`, `hyper`, `axum`, SSE or streamable-HTTP crates).

## Implementation gates (the MCP server is not done until all three pass, in CI)

| Gate | From | What must be shown |
|---|---|---|
| **G1 — both eras** | Q-C | Real stdio calls from a 2026-07-28 client (per-request version, `server/discover`) **and** a 2025-11-25 client (`initialize` handshake): listing tools and calling a scalar, a list and a choice calculation. |
| **G2 — tool-name rules** | Q-D | `check_descriptor` rejects calculation IDs longer than **128 characters** or with characters outside MCP's set (`A–Z a–z 0–9 _ - .`). CivKit's ID rule (lower-case letters, digits, `_`, three dot-separated segments) is already a subset; the length limit is new. Tests first. |
| **G3 — round trip** | Q-E | In both eras: `structuredContent` equals `Registry::call_json`'s output for the same request; the text block holds the same JSON; a contract error keeps `isError: true` and its envelope (`code`, `input`, `index`) intact. |

These sit beside the existing Phase 3 criteria (MCP Inspector and a real client, fully offline, stdio only).

## Consequences

- API_DESIGN §10 gains a short mapping section pointing here.
- IMPLEMENTATION_PLAN Phase 3 and ACCEPTANCE_CRITERIA Phase 3 carry the three gates.
- MCP resources and prompts (FEATURE_PROPOSALS FP-19), drawing tools (Phase 4) and any write tool (S-5) are outside
  this decision and need their own.
- The remaining Phase 3 precondition is the Python side: supported versions and the wheel approach.
