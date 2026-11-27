# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 22:45 IST · last edited 2026-11-27 22:53 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)
"""Drawing analysis from Python (D-28 / ADR 0016 Q-A, API_DESIGN §13): `civkit.drawing.analyze` and
`civkit.drawing.schedule_csv`. Exact parity with Rust and MCP for every fixture is in tests/parity."""

import json
import os
import tempfile
import unittest
from pathlib import Path

import civkit
from civkit import _civkit

FIXTURES = Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "dxf"
F1 = FIXTURES / "f1_site_plan_m.dxf"
F5 = FIXTURES / "f5_blocks_inserts.dxf"

FORMULAS = (
    "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n9\n$INSUNITS\n70\n6\n0\nENDSEC\n"
    "0\nSECTION\n2\nENTITIES\n"
    + "".join(
        f"0\nLINE\n5\n{k + 10:X}\n8\n{name}\n10\n0\n20\n0\n30\n0\n11\n1\n21\n0\n31\n0\n"
        for k, name in enumerate(["=cmd|' /C calc'!A0", "+1", "-2", "@x"])
    )
    + "0\nENDSEC\n0\nEOF\n"
).encode()


class Analyze(unittest.TestCase):
    def test_a_path_a_string_and_bytes_give_the_same_report(self):
        by_path = civkit.drawing.analyze(F1)
        self.assertEqual(by_path, civkit.drawing.analyze(str(F1)))
        self.assertEqual(by_path, civkit.drawing.analyze(F1.read_bytes()))
        self.assertEqual(by_path, civkit.drawing.analyze(bytearray(F1.read_bytes())))
        self.assertEqual((by_path["report"], by_path["version"]), ("drawing.analyze", "1.0.0"))
        boundary = next(l for l in by_path["layers"] if l["name"] == "BOUNDARY")
        self.assertEqual(boundary["drawing_units"]["polyline_closed_area"], 2550.0)

    def test_the_rust_layer_returns_the_rust_text(self):
        ok, text = _civkit.drawing_json(F1.read_bytes(), "layer")
        self.assertTrue(ok)
        self.assertEqual(json.loads(text), civkit.drawing.analyze(F1))

    def test_detail(self):
        self.assertNotIn("entities", civkit.drawing.analyze(F5))
        rows = civkit.drawing.analyze(F5, detail="entity")["entities"]
        self.assertEqual(len(rows), 32)
        with self.assertRaises(civkit.CivKitError) as e:
            civkit.drawing.analyze(F5, detail="everything")
        self.assertEqual((e.exception.code, e.exception.input), ("INVALID_VALUE", "detail"))

    def test_an_unreadable_drawing_is_a_civkit_error(self):
        with self.assertRaises(civkit.CivKitError) as e:
            civkit.drawing.analyze(b"0\nSECTION\n2\nENTITIES\n0\nLINE\n10\nx\n")
        self.assertEqual(e.exception.code, "DRAWING_UNREADABLE")

    def test_a_missing_file_is_python_s_own_error(self):
        with self.assertRaises(FileNotFoundError):
            civkit.drawing.analyze(FIXTURES / "no-such-drawing.dxf")

    def test_other_types_are_refused(self):
        with self.assertRaises(TypeError):
            civkit.drawing.analyze(12)  # type: ignore[arg-type]


class Schedule(unittest.TestCase):
    def test_the_schedule_is_the_rust_text(self):
        csv = civkit.drawing.schedule_csv(F1)
        self.assertTrue(csv.startswith("layer,kind,count,length,area,unit,length_m,area_m2,incomplete,omitted,"))
        ok, text = _civkit.drawing_csv(F1.read_bytes())
        self.assertTrue(ok)
        self.assertEqual(csv, text)
        self.assertIn("BOUNDARY,polyline_closed,1,200,2550,m,200,2550,false,0,1.0.0,", csv)

    def test_formula_like_layer_names_are_neutralised(self):
        rows = civkit.drawing.schedule_csv(FORMULAS).splitlines()[1:]
        first_cells = sorted(r.split(",")[0] for r in rows)
        self.assertEqual(first_cells, ["'+1", "'-2", "'=cmd|' /C calc'!A0", "'@x"])
        # The JSON report keeps the names as the drawing states them: it is data, not a spreadsheet.
        names = sorted(l["name"] for l in civkit.drawing.analyze(FORMULAS)["layers"])
        self.assertEqual(names, ["+1", "-2", "=cmd|' /C calc'!A0", "@x"])


class NothingIsWritten(unittest.TestCase):
    def test_no_file_appears(self):
        with tempfile.TemporaryDirectory() as d:
            here = os.getcwd()
            os.chdir(d)
            try:
                civkit.drawing.analyze(F1)
                civkit.drawing.schedule_csv(F1)
            finally:
                os.chdir(here)
            self.assertEqual(os.listdir(d), [])


if __name__ == "__main__":
    unittest.main()
