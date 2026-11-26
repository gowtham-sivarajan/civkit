# Proposal: the MCP SDK and server surface (D-13)

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-26 23:05 IST · last edited 2026-11-26 23:15 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** **Accepted by the maintainer, 2026-11-26: yes to Q-A through Q-G**, with two implementation gates added (both protocol eras tested; MCP tool-name rules enforced). Recorded as [ADR 0013](decisions/0013-mcp-sdk-and-surface.md), which is the source of truth; the text below is unchanged from the proposal.
>
> Facts checked on 2026-11-26 against the MCP specification revision 2026-07-28
> (https://modelcontextprotocol.io/specification/2026-07-28, pages: tools, transports, stdio, lifecycle, schema) and the
> official Rust SDK (https://github.com/modelcontextprotocol/rust-sdk: README, `Cargo.toml` manifests, releases), plus
> `cargo info rmcp`. Labels: **[V]** read on the page; **[S]** from a summarised part of a long page; **[I]** my inference.

## 1. What D-13 has to decide

D-13 as registered: *"Official `rmcp` crate, stdio only; pin version at Phase 3 start."* API_DESIGN §10 already fixes the
behaviour: stdio only; one tool per calculation plus `civkit_list`, `civkit_describe`, `civkit_call`; `--tools
per-calc|generic|both`, default `both`; every tool read-only; results carry the full contract response. What remains is
**which crate and version, with which features, and how the CivKit contract maps onto MCP results**.

## 2. The SDK: `rmcp`

| Fact | Value | Label |
|---|---|---|
| Latest version | **3.12.0**, released 2026-11-26 (3.11.0 on 2026-11-17, 3.10.0 on 2026-11-12) | [V] |
| Licence | Apache-2.0 (passes `deny.toml`) | [V] |
| Minimum Rust | 1.88 (CivKit's MSRV is 1.98) | [V] |
| Edition | 2024 | [V] |
| Default features | `base64`, `chrono`, `macros`, `server` | [V] |
| Stdio server | `server` plus `transport-io` (`transport-io` = async read/write + `tokio/io-std` + `tokio/net`) | [V] |
| Network crates | `reqwest`, `hyper`, `axum`, SSE and streamable HTTP are **optional and off by default**; none is needed for stdio | [V] |
| Always-on dependencies | `serde`, `serde_json`, `tokio` (`sync`, `macros`, `rt`, `time`), `futures`, `tracing`, `thiserror`, `indexmap`, `jiff`, `sha2`, unicode crates and others | [V] |
| Spec support claim | "implements the stable MCP 2026-07-28 specification while remaining fully compatible with the 2025-11-25 release and earlier" | [V] README; **untested** |

## 3. What the 2026-07-28 specification requires of a stdio server

- **Stdout carries only MCP messages**, newline-delimited UTF-8 JSON-RPC; logging goes to **stderr** [V]. The server
  should exit when stdin closes [V].
- **Version eras** [V]: 2026-07-28 removed the `initialize` handshake; each request carries its version. A modern-only
  server **fails** with clients on 2025-11-25 or earlier; a *dual-era* server works with both. Servers must implement
  `server/discover` [V].
- **Tool names** [V]: 1–128 characters from `A–Z a–z 0–9 _ - .` (a SHOULD). **CivKit IDs fit unchanged**
  (`fluids.open_channel.manning_trapezoidal`).
- **Schemas** [V]: `inputSchema` must be a JSON Schema object (2020-12 by default); with `outputSchema`, the server
  **must** return `structuredContent` that conforms; it **should** also put the same JSON in a text block for older clients.
- **Errors** [V]: execution errors, including invalid arguments, are a normal result with `isError: true`, which the model
  sees; protocol errors (for example an unknown tool) are JSON-RPC errors.
- **Annotations** [S]: `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint` are hints, and clients must
  treat them as untrusted unless the server is trusted.

## 4. The recommended choice

```toml
# integrations/mcp/Cargo.toml (crate civkit-mcp, a binary; a workspace member outside crates/)
[dependencies]
civkit-core = { path = "../../crates/civkit-core" }
rmcp = { version = "=3.12.0", default-features = false, features = ["server", "transport-io"] }
tokio = { version = "1", default-features = false, features = ["rt", "io-std", "macros"] }
serde_json = "1"
```

*(Illustrative: written only after D-13 is accepted. Exact tokio features are settled when the code is written.)*

- **No `macros`.** `rmcp`'s macros define tools at compile time; CivKit's tools are generated at run time from the
  registry, so the server implements the handler directly. This also keeps `chrono` and `base64` out.
- **Exact pin (`=3.12.0`)** plus the committed `Cargo.lock`. Dependabot proposes upgrades; each must pass CI and the
  parity tests.
- **`transport-io` turns on `tokio/net`** at compile time. No socket is opened (stdio only). The dependency-rule check
  is extended so that `civkit-mcp`'s tree may contain no HTTP crate (`reqwest`, `hyper`, `axum`, SSE).

## 5. How the contract maps onto MCP

| CivKit | MCP |
|---|---|
| Calculation ID | Per-calculation tool **name**, verbatim; descriptor `title` → tool `title`; `summary` → `description` |
| Descriptor inputs | `inputSchema` generated from the descriptor: each input is `{ value, unit, negative? }`; lists become arrays with `minItems` (and `maxItems`, from the 10 000 limit); choices become `enum`; required inputs are `required` |
| Generic tools | `civkit_list`, `civkit_describe`, `civkit_call` (API_DESIGN §10) |
| Success | `structuredContent` = the response exactly as `Registry::call_json` returns it; the same JSON in a text block |
| Contract error (`UNIT_MISMATCH`, `OUT_OF_DOMAIN`, …) | `isError: true`; the error envelope (with `code`, `input`, `index`) as `structuredContent` and as text |
| `outputSchema` | an object schema with `oneOf` [response, error envelope], so both successes and contract errors conform |
| Unknown tool, malformed request | JSON-RPC protocol error (rmcp) |
| Annotations | `readOnlyHint: true`, `openWorldHint: false` (every tool reads; none touches the outside world) |

The parity rule: for any request, `structuredContent` must equal the parsed output of `Registry::call_json`. This is the
check in IMPLEMENTATION_PLAN Phase 3 ("real stdio MCP calls"; list and choice inputs, units, trace fields, `index`).

## 6. Questions for the maintainer

| # | Question | Recommendation |
|---|---|---|
| **Q-A** | Pin `rmcp` **=3.12.0** (released today), or **=3.11.0** (nine days old; includes "reject unadvertised capability methods by default")? | **3.12.0**, re-checked on the day the code starts; drop to 3.11.0 if 3.12.0 shows any regression in our tests. |
| **Q-B** | Features: `default-features = false`, `server` + `transport-io` only, no `macros`. | **As proposed.** |
| **Q-C** | Support both MCP eras (2026-07-28 and 2025-11-25 clients), relying on rmcp's dual-era support, and **test both**? | **Yes.** Many clients may lag the new revision; a modern-only server would fail with them. If rmcp cannot serve both, come back to this decision. |
| **Q-D** | Tool names = calculation IDs verbatim; generic tools as in API_DESIGN §10; `--tools both` default. | **As proposed.** |
| **Q-E** | Result mapping in §5: full contract response as `structuredContent` plus text; contract errors as `isError: true` with the envelope; `outputSchema` = `oneOf` [response, error]. | **As proposed.** It keeps the contract byte-for-byte and lets clients validate. |
| **Q-F** | Annotations `readOnlyHint: true`, `openWorldHint: false`. | **As proposed.** |
| **Q-G** | Crate `civkit-mcp` in `integrations/mcp`, a workspace member; logging only to stderr; the dependency check extended to forbid HTTP crates in its tree. | **As proposed.** |

Not in D-13: MCP resources and prompts (FEATURE_PROPOSALS FP-19), drawing tools (Phase 4), any write tool (S-5). Those
come later, each with its own proposal.

## 7. After acceptance

1. Record D-13 as Accepted with the pinned version in a new ADR; update the D-13 row in DECISIONS.md.
2. Propose the Python side (supported versions, wheel approach) the same way: Phase 3 precondition.
3. Then code, tests first: parity tests against `Registry::call_json`, real stdio calls (one modern and one legacy
   client, plus MCP Inspector), offline.
