<!-- Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:46 IST · last edited 2026-11-25 10:46 IST -->

## What this changes

<!-- One or two sentences. Link the issue: "Closes #123". -->

## Checklist

- [ ] I have read [CONTRIBUTING.md](https://github.com/gowtham-sivarajan/civkit/blob/main/CONTRIBUTING.md).
- [ ] The engine (`crates/`) gains no dependency on AutoCAD/.NET, Python, the network or AI services (dependency rule, [ARCHITECTURE.md](https://github.com/gowtham-sivarajan/civkit/blob/main/docs/ARCHITECTURE.md)).
- [ ] Nothing here needs an internet connection, an account or telemetry ([OFFLINE_POLICY.md](https://github.com/gowtham-sivarajan/civkit/blob/main/docs/OFFLINE_POLICY.md)).
- [ ] `cargo fmt`, `cargo clippy` and the tests pass locally (once code exists).
- [ ] Docs are updated where behaviour changed.

### If this adds or changes a calculation

- [ ] Descriptor complete: ID, version, inputs/outputs with dimensions, references, assumptions, validity.
- [ ] At least one reference case from an **openly licensed or public-domain source** (D-18), cited with edition and page.
- [ ] Expected result independently verified (hand re-derivation or a second source).
- [ ] Edge cases and invalid inputs tested (correct error codes).
- [ ] If a result changed for the same input, the calculation version is bumped ([API_DESIGN.md §4](https://github.com/gowtham-sivarajan/civkit/blob/main/docs/API_DESIGN.md)).

### AI assistance

- [ ] If an AI tool wrote any part of this, the files carry the provenance line and the commit has a `Co-Authored-By` trailer ([CONTRIBUTING.md](https://github.com/gowtham-sivarajan/civkit/blob/main/CONTRIBUTING.md)).
