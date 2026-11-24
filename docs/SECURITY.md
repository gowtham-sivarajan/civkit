# Security

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:29 IST · last edited 2026-11-25 00:40 IST

> **Status:** Draft. No released versions yet.

## 1. Reporting a vulnerability

Report vulnerabilities through GitHub's **private vulnerability reporting** on this repository
(Security tab → "Report a vulnerability"), not in public issues. Expect an acknowledgement within 7 days.

## 2. Supported versions

None yet. Before 1.0, only the latest release receives security fixes.

## 3. Threat model

CivKit runs locally and has no server, so the main risks come from **untrusted inputs** and **the supply chain**.

| Entry point | Threat | Mitigation |
|---|---|---|
| DXF files (any interface) | Malformed or hostile files causing crashes, hangs or memory exhaustion | Hardened parser; size, entity-count and nesting limits; fuzzing (`cargo-fuzz`) from Phase 4; no code execution from drawing content |
| JSON requests (Python, MCP, FFI) | Malformed input, huge payloads | Schema validation, size limits, structured errors — never a crash |
| FFI boundary | Rust panic unwinding into C#; memory misuse | Panics caught and returned as errors; clear ownership rules (`civkit_free`) — see [API_DESIGN.md §9](API_DESIGN.md#9-ffi-cratescivkit-ffi) |
| MCP server | Prompt injection: text inside a drawing (layer names, notes) reaching the AI client as instructions | Drawing-sourced text is returned as clearly labelled data; all 0.1 tools are read-only; any future write tool requires confirmation in the host |
| MCP server | Reading files the user did not intend to expose | Drawing tools only read paths inside user-configured locations |
| AutoCAD / Civil 3D plugin | Plugin DLL hijacking or tampering | Install to AutoCAD trusted locations (respecting AutoCAD's secure-load settings); release checksums; code signing to be investigated (must be free for the project) |
| Desktop app | Remote content or script injection | No remote content; strict CSP; minimal Tauri capabilities (see [OFFLINE_POLICY.md](OFFLINE_POLICY.md)) |

## 4. Supply chain

- Lockfiles (`Cargo.lock`, Python and npm lockfiles) are committed.
- `cargo-audit` and `cargo-deny` (advisories, licences, bans) run in CI; `pip-audit` and `npm audit` for their parts.
- Keep dependencies few; every new dependency is justified in its pull request.
- GitHub Actions pinned to commit SHAs.
- Releases publish SHA-256 checksums.

## 5. Privacy

No telemetry and no data leaves the machine ([OFFLINE_POLICY.md](OFFLINE_POLICY.md)).
