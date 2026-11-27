# CivKit notebooks

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-27 11:21 IST · last edited 2026-11-27 11:21 IST

> **Disclaimer.** CivKit results support, and do not replace, the judgement of a qualified engineer.

Example notebooks for the `civkit` Python package. They need only `civkit` and Jupyter, and run offline.

| Notebook | Shows |
|---|---|
| [01_getting_started.ipynb](01_getting_started.ipynb) | Listing and describing calculations; the three quantity forms; output units and `.to()`; the trace; errors; the JSON contract |
| [02_surveying.ipynb](02_surveying.ipynb) | Compass-rule traverse adjustment; height-of-instrument and rise-and-fall levelling; closure adjustment |
| [03_beam_and_earthwork.ipynb](03_beam_and_earthwork.ipynb) | Simply supported beam, including its conditional input; earthwork volumes with both methods |

The examples are the published cases CivKit is tested against (`tests/reference/`). Per-calculation reference pages
are in [docs/calculations](../../docs/calculations/README.md).

The notebooks are stored without outputs. `bindings/python/tests/test_notebooks.py` runs every code cell in order
in each Python CI job, so they keep working as the package changes.
