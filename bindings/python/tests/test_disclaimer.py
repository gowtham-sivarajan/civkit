# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-28 14:10 IST · last edited 2026-11-28 14:10 IST
"""The disclaimer (ENGINEERING_STANDARDS §7) in the Python interface: package and public API documentation, and the
display of a Result or a drawing report, Jupyter included. The JSON (``to_dict()``, the report dict) is unchanged."""

import html
import json
import unittest
from pathlib import Path

import civkit
from civkit import _civkit

ROOT = Path(__file__).resolve().parents[3]
TEXT = "CivKit results support, and do not replace, the judgement of a qualified engineer."
F1 = ROOT / "tests" / "fixtures" / "dxf" / "f1_site_plan_m.dxf"
MANNING = dict(bottom_width="1.5 m", side_slope=2.0, depth="0.6 m", manning_n=0.013, bed_slope=0.002)


class TheText(unittest.TestCase):
    def test_it_is_the_engine_s_one_string(self):
        self.assertEqual(civkit.DISCLAIMER, _civkit.DISCLAIMER)
        self.assertEqual(civkit.DISCLAIMER, TEXT)
        self.assertIn("DISCLAIMER", civkit.__all__)

    def test_it_matches_the_generated_documentation_and_the_stub(self):
        import importlib.util
        for name, path in (("gen_calc_docs", ROOT / "tools" / "gen_calc_docs.py"),
                           ("gen_stub", ROOT / "bindings" / "python" / "tools" / "gen_stub.py")):
            spec = importlib.util.spec_from_file_location(name, path)
            gen = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(gen)
            with self.subTest(generator=name):
                self.assertEqual(gen.DISCLAIMER, civkit.DISCLAIMER)


class Documentation(unittest.TestCase):
    def test_package_and_public_functions(self):
        for obj in (civkit, civkit.call, civkit.describe, civkit.list, civkit.drawing,
                    civkit.drawing.analyze, civkit.drawing.schedule_csv):
            with self.subTest(obj=getattr(obj, "__name__", obj)):
                self.assertIn(TEXT, obj.__doc__ or "")

    def test_every_typed_calculation_function(self):
        for d in civkit.list():
            *path, name = d["id"].split(".")
            node = civkit
            for part in path:
                node = getattr(node, part)
            with self.subTest(id=d["id"]):
                self.assertIn(TEXT, getattr(node, name).__doc__)


class ResultDisplay(unittest.TestCase):
    def setUp(self):
        self.r = civkit.call("fluids.open_channel.manning_trapezoidal", **MANNING)

    def test_repr_and_rich_display_show_it(self):
        self.assertIn(TEXT, repr(self.r))
        self.assertIn(html.escape(TEXT), self.r._repr_html_())

    def test_the_json_is_unchanged(self):
        d = self.r.to_dict()
        self.assertNotIn("disclaimer", json.dumps(d).lower())
        request = self.r.request
        self.assertEqual(d, json.loads(_civkit.call_json(json.dumps(request))))


class DrawingDisplay(unittest.TestCase):
    def setUp(self):
        self.report = civkit.drawing.analyze(F1)

    def test_repr_and_rich_display_show_it(self):
        self.assertIn(TEXT, repr(self.report))
        self.assertIn(html.escape(TEXT), self.report._repr_html_())

    def test_the_report_is_still_the_engine_s_dict(self):
        self.assertIsInstance(self.report, dict)
        ok, text = _civkit.drawing_json(F1.read_bytes(), "layer")
        self.assertTrue(ok)
        self.assertEqual(self.report, json.loads(text))
        self.assertEqual(json.dumps(self.report, sort_keys=True), json.dumps(json.loads(text), sort_keys=True))
        self.assertNotIn("disclaimer", self.report)

    def test_drawing_strings_are_escaped_in_the_rich_display(self):
        dxf = F1.read_bytes().replace(b"\r\nBOUNDARY\r\n", b"\r\n<b>X</b>\r\n")
        self.assertIn(b"<b>X</b>", dxf)
        shown = civkit.drawing.analyze(dxf)._repr_html_()
        self.assertNotIn("<b>X</b>", shown)
        self.assertIn("&lt;b&gt;X&lt;/b&gt;", shown)


if __name__ == "__main__":
    unittest.main()
