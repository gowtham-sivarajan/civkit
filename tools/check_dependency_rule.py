#!/usr/bin/env python3
# Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 11:20 IST · last edited 2026-11-25 11:20 IST
"""Checks the dependency rule of docs/ARCHITECTURE.md §2.

1. No package under crates/ has a path dependency outside crates/.
2. No package under crates/ depends, directly or transitively, on a Python, MCP, AI, network,
   async-runtime or GUI crate.

Uses only `cargo metadata` and the Python standard library. Exit code 1 on any violation.
Run from the repository root:  python tools/check_dependency_rule.py
"""

import json
import subprocess
import sys
from pathlib import Path

# Crates the engine must never reach. Prefix entries end with '*'.
FORBIDDEN = {
    # Python
    "pyo3", "pyo3-*", "numpy",
    # MCP and AI services
    "rmcp", "rmcp-*", "async-openai", "openai*", "anthropic*", "ollama*",
    # Network clients, servers and TLS
    "reqwest", "ureq", "curl", "isahc", "attohttpc", "surf", "hyper", "h2", "h3", "axum", "warp",
    "actix-web", "rustls", "native-tls", "openssl", "tungstenite", "tokio-tungstenite",
    # Async runtimes (the engine is synchronous and offline)
    "tokio", "async-std", "smol",
    # GUI / host frameworks
    "tauri", "tauri-*", "wry",
}


def forbidden(name: str) -> bool:
    return any(name == f or (f.endswith("*") and name.startswith(f[:-1])) for f in FORBIDDEN)


def main() -> int:
    root = Path.cwd().resolve()
    crates_dir = root / "crates"
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--all-features", "--locked"],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        print("cargo metadata failed:\n" + result.stderr)
        return 1
    meta = json.loads(result.stdout)
    packages = {p["id"]: p for p in meta["packages"]}
    engine = [pid for pid in meta["workspace_members"]
              if Path(packages[pid]["manifest_path"]).resolve().is_relative_to(crates_dir)]
    errors = []

    # Rule 1: path dependencies stay inside crates/.
    for pid in engine:
        pkg = packages[pid]
        for dep in pkg["dependencies"]:
            if dep.get("path") and not Path(dep["path"]).resolve().is_relative_to(crates_dir):
                errors.append(f"{pkg['name']} depends on '{dep['name']}' at {dep['path']}, outside crates/")

    # Rule 2: no forbidden crate anywhere in an engine crate's dependency graph (normal and build deps).
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    for pid in engine:
        seen, stack, parent = {pid}, [pid], {}
        while stack:
            current = stack.pop()
            for dep in nodes[current]["deps"]:
                kinds = {k.get("kind") for k in dep["dep_kinds"]}
                if kinds == {"dev"}:
                    continue  # test-only dependencies do not ship in the engine
                if dep["pkg"] in seen:
                    continue
                seen.add(dep["pkg"])
                parent[dep["pkg"]] = current
                stack.append(dep["pkg"])
                name = packages[dep["pkg"]]["name"]
                if forbidden(name):
                    chain, node = [name], current
                    while node != pid:
                        chain.append(packages[node]["name"])
                        node = parent[node]
                    chain.append(packages[pid]["name"])
                    errors.append("forbidden dependency: " + " -> ".join(reversed(chain)))

    names = ", ".join(sorted(packages[p]["name"] for p in engine))
    if errors:
        print("Dependency rule violated (docs/ARCHITECTURE.md §2):")
        for e in errors:
            print("  - " + e)
        return 1
    print(f"Dependency rule OK for {len(engine)} engine crates: {names}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
