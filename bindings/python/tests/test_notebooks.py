# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 11:21 IST · last edited 2026-11-27 11:21 IST
"""The example notebooks in integrations/jupyter (IMPLEMENTATION_PLAN Phase 3 task 4) run from top to bottom with the
installed package, show the disclaimer, and are stored without outputs.

The code cells are executed in order in one namespace, without Jupyter, so this runs in every Python CI job."""

import contextlib
import io
import json
import unittest
from pathlib import Path

NOTEBOOKS = Path(__file__).resolve().parents[3] / "integrations" / "jupyter"
DISCLAIMER = "CivKit results support, and do not replace, the judgement of a qualified engineer."


def notebooks():
    found = sorted(NOTEBOOKS.glob("*.ipynb"))
    if not found:
        raise AssertionError(f"no notebooks under {NOTEBOOKS}")
    return found


class Notebooks(unittest.TestCase):
    def test_there_are_examples_for_every_domain(self):
        names = [p.stem for p in notebooks()]
        self.assertGreaterEqual(len(names), 3, names)
        text = " ".join(p.read_text(encoding="utf-8") for p in notebooks())
        import civkit

        for d in civkit.list():
            self.assertIn(d["id"].rsplit(".", 1)[-1], text, f"{d['id']} appears in a notebook")

    def test_each_notebook_runs_top_to_bottom(self):
        for path in notebooks():
            with self.subTest(path.name):
                nb = json.loads(path.read_text(encoding="utf-8"))
                namespace = {"__name__": "__notebook__"}
                for n, cell in enumerate(nb["cells"]):
                    if cell["cell_type"] != "code":
                        continue
                    source = "".join(cell["source"])
                    with contextlib.redirect_stdout(io.StringIO()):
                        try:
                            exec(compile(source, f"{path.name} cell {n}", "exec"), namespace)
                        except Exception as e:  # noqa: BLE001 (report which cell failed)
                            self.fail(f"{path.name} cell {n} raised {type(e).__name__}: {e}\n{source}")

    def test_each_notebook_shows_the_disclaimer_first(self):
        for path in notebooks():
            with self.subTest(path.name):
                first = json.loads(path.read_text(encoding="utf-8"))["cells"][0]
                self.assertEqual(first["cell_type"], "markdown")
                self.assertIn(DISCLAIMER, "".join(first["source"]))

    def test_notebooks_are_stored_without_outputs(self):
        for path in notebooks():
            with self.subTest(path.name):
                for cell in json.loads(path.read_text(encoding="utf-8"))["cells"]:
                    if cell["cell_type"] == "code":
                        self.assertEqual(cell["outputs"], [])
                        self.assertIsNone(cell["execution_count"])


if __name__ == "__main__":
    unittest.main()
