# civkit-mcp

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-27 00:24 IST · last edited 2026-11-27 00:24 IST

CivKit's calculations as [MCP](https://modelcontextprotocol.io) tools, over **stdio only** (D-13,
[ADR 0013](../../planning/decisions/0013-mcp-sdk-and-surface.md)). The server opens no socket, reads nothing from the
network, and exits when its client closes stdin.

```text
civkit-mcp [--tools per-calc|generic|both]     # default: both
```

Build: `cargo build --release -p civkit-mcp` (the binary is `target/release/civkit-mcp`, `.exe` on Windows).

## Tools

| Tool | Arguments | Result |
|---|---|---|
| one per calculation, named by its ID (`fluids.open_channel.manning_trapezoidal`) | each input as `{ "value": …, "unit": … }`, plus optional `output_units` | the CivKit response |
| `civkit_list` | none | `{ "calculations": [ { id, version, title, summary } ] }` |
| `civkit_describe` | `id`, optional `version` | the full descriptor |
| `civkit_call` | a full CivKit request: `id`, optional `version`, `inputs`, optional `output_units` | the CivKit response |

- **Success:** `structuredContent` is the response exactly as `Registry::call_json` returns it; the same JSON is in a
  text block.
- **Contract error** (`MISSING_UNIT`, `OUT_OF_DOMAIN`, …): `isError: true`, with the error envelope (`code`, `input`,
  `index`, `message`) as `structuredContent` and as text, so the model can correct its input.
- **Unknown tool:** a JSON-RPC protocol error.
- Every tool is annotated `readOnlyHint: true`, `openWorldHint: false`.

## Protocol versions

Both eras are served: 2025-11-25 clients (the `initialize` handshake) and 2026-07-28 clients (the version in each
request's `_meta`, plus `server/discover`).

## Client configuration

Most clients take a command and arguments. For example, a JSON configuration such as:

```json
{ "mcpServers": { "civkit": { "command": "C:/path/to/civkit-mcp.exe", "args": ["--tools", "both"] } } }
```

## Tests

`cargo test -p civkit-mcp` spawns the binary and speaks to it over real stdio in both eras (gates G1 and G3 of ADR
0013): it lists the tools, and calls every reference case through its own tool and through `civkit_call`, comparing
each result with `Registry::call_json`. Gate G2 (tool-name rules) is in `crates/civkit-core/tests/contract.rs`.
