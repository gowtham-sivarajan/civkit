# Security

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:29 IST · last edited 2026-11-27 22:53 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

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
| DXF files (any interface) | Malformed or hostile files causing crashes, hangs or memory exhaustion | CivKit's own parser ([ADR 0015](../planning/decisions/0015-dxf-reader.md)): input-size limit, a heap budget that every allocation reserves from (parsing and expansion share it), per-item caps, nesting limit, no recursion; fuzzing (`cargo-fuzz`); no code execution from drawing content |
| JSON requests (Python, MCP, FFI) | Malformed input, huge payloads | Schema validation, size limits, structured errors — never a crash |
| FFI boundary | Rust panic unwinding into C#; memory misuse | Panics caught and returned as errors; clear ownership rules (`civkit_free`) — see [API_DESIGN.md §9](API_DESIGN.md#9-ffi-cratescivkit-ffi) |
| MCP server | Prompt injection: text inside a drawing (layer names, notes) reaching the AI client as instructions | Drawing-sourced text is returned as clearly labelled data; all 0.1 tools are read-only; any future write tool requires confirmation in the host |
| MCP server | Reading files the user did not intend to expose | The drawing tool exists only with `--drawing-root`; it reads only absolute paths whose real location (symbolic links and junctions resolved) is a file inside a root, with one refusal message for every case (no existence oracle), and at most the input limit plus one byte. Residual: a hard link inside a root to a file elsewhere is read (making one needs access to that file already), and a directory swapped for a link between the check and the open is not detected |
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
