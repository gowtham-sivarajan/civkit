# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 11:21 IST · last edited 2026-11-27 11:21 IST
"""The generated calculation pages (docs/calculations) match the installed calculations, and carry what
ENGINEERING_STANDARDS §2 item 7 asks: formula, symbols, references, assumptions, limits, example; plus the disclaimer
(§7) and conditional requirements shown prominently (IMPLEMENTATION_PLAN Phase 3 task 5)."""

import importlib.util
import unittest
from pathlib import Path

import civkit

TOOL = Path(__file__).resolve().parents[3] / "tools" / "gen_calc_docs.py"
_spec = importlib.util.spec_from_file_location("gen_calc_docs", TOOL)
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)


class CalculationPages(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.pages = gen.render_all()

    def test_the_pages_have_not_drifted(self):
        for name, text in self.pages.items():
            with self.subTest(name):
                path = gen.OUT / name
                self.assertTrue(path.exists(), f"missing {path}: run `python tools/gen_calc_docs.py`")
                self.assertEqual(path.read_text(encoding="utf-8"), text, "run `python tools/gen_calc_docs.py`")
        on_disk = {p.name for p in gen.OUT.glob("*.md")}
        self.assertEqual(on_disk, set(self.pages), "no stale pages")

    def test_one_page_per_calculation_and_an_index(self):
        self.assertEqual(set(self.pages), {f"{d['id']}.md" for d in civkit.list()} | {"README.md"})
        for d in civkit.list():
            self.assertIn(f"]({d['id']}.md)", self.pages["README.md"])

    def test_every_page_has_the_required_sections(self):
        for d in civkit.list():
            text = self.pages[f"{d['id']}.md"]
            with self.subTest(d["id"]):
                for heading in ["## Inputs", "## Outputs", "## Method", "## Validity and limits", "## Assumptions",
                                "## References", "## Worked example", "## Use it"]:
                    self.assertIn(heading, text)
                self.assertIn(gen.DISCLAIMER, text)
                for i in d["inputs"]:
                    self.assertIn(f"| `{i['name']}` |", text)
                for r in d["references"]:
                    self.assertIn(r, text)
                self.assertIn("1. `", text, "the method lists at least one trace step")

    def test_the_conditional_requirement_is_prominent(self):
        beam = self.pages["structural.beam.simply_supported.md"]
        self.assertIn("**conditional: required when point_load > 0**", beam)
        self.assertIn("> **Conditional requirement:** `point_load_position` is required when `point_load > 0`.", beam)

    def test_per_item_steps_appear_once(self):
        traverse = self.pages["surveying.traverse.compass_rule.md"]
        self.assertIn("for each leg:", traverse)
        self.assertNotIn("leg 2:", traverse)


if __name__ == "__main__":
    unittest.main()
