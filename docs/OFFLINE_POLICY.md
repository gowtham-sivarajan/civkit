# Offline Policy

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 00:28 IST · last edited 2026-11-28 14:15 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

> **Status:** Draft. Turns the proposal's local-first principles into rules that can be checked.

## 1. Scope

This policy covers CivKit **at run time**: the engine, Python package, MCP server, AutoCAD/Civil 3D plugins,
MATLAB wrapper and desktop app. **Build time** is different: downloading dependencies from crates.io, PyPI,
npm and NuGet while developing is allowed.

## 2. Rules

1. **No feature requires a network connection.** Every capability works with the network unplugged.
2. **No telemetry.** No analytics, usage statistics, crash uploads or "phone home" of any kind.
3. **No accounts, sign-in or licence checks.**
4. **No automatic update checks.** If an update notice is ever added, it is opt-in, off by default, and
   requires a decision record.
5. **Documentation is bundled.** Help, calculation docs and references are available offline.
6. **MCP runs over stdio only.** The MCP server opens no network ports.
7. **The desktop app loads no remote content.** Tauri is configured with a strict Content Security Policy;
   no remote URLs, fonts, scripts or images.
8. **No data fetched at import or start-up.** The Python package and plugins do not download anything.
9. **Data stays local.** CivKit writes only where the user asks (exports) or to its own local data folder
   (desktop history). Export formats are open: CSV, JSON, DXF.

## 3. Enforcement

- **Dependency bans:** `cargo-deny` configuration bans HTTP/network client crates (e.g. `reqwest`, `hyper`,
  `ureq`) from everything under `crates/`, `bindings/` and `integrations/`.
- **Offline CI job:** the test suite runs with networking disabled after dependencies are fetched. In `ci.yml`, the
  `offline` job fetches and builds, checks that a network namespace with no interfaces cannot reach the network, and
  then runs `cargo test --offline` for the whole workspace inside it. The Python workflow installs and runs the wheel
  offline (gates P1 and P5).
- **Review checklist:** every pull request answers "does this add any network access?"
- **Desktop app:** Tauri capability configuration grants no network permissions; reviewed on every change.

## 4. Exceptions

Any feature that needs the network must be: optional, off by default, clearly labelled in the interface,
documented here, and approved through a decision record in [planning/DECISIONS.md](../planning/DECISIONS.md).
None exist today.
