# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-27 11:21 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)
"""ADR 0014 Q-E: the type stub matches the installed calculations, and agrees with the generated functions."""

import ast
import importlib.util
import inspect
import unittest
from pathlib import Path

import civkit

TOOL = Path(__file__).resolve().parents[1] / "tools" / "gen_stub.py"
_spec = importlib.util.spec_from_file_location("gen_stub", TOOL)
gen_stub = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen_stub)


class Stub(unittest.TestCase):
    def test_the_stub_has_not_drifted(self):
        current = gen_stub.STUB.read_text(encoding="utf-8")
        self.assertEqual(current, gen_stub.render(civkit.list()), "run `python tools/gen_stub.py`")

    def test_the_stub_is_valid_python(self):
        ast.parse(gen_stub.STUB.read_text(encoding="utf-8"))

    def test_each_stub_signature_names_the_runtime_parameters(self):
        tree = ast.parse(gen_stub.STUB.read_text(encoding="utf-8"))
        stubs = {
            (cls.name, node.name): [a.arg for a in node.args.kwonlyargs]
            for cls in tree.body if isinstance(cls, ast.ClassDef)
            for node in cls.body if isinstance(node, ast.FunctionDef) and node.decorator_list
        }
        for d in civkit.list():
            domain, topic, method = d["id"].split(".")
            fn = getattr(getattr(getattr(civkit, domain), topic), method)
            self.assertEqual(stubs[(f"_{domain}_{topic}", method)], list(inspect.signature(fn).parameters), d["id"])

    def test_a_conditional_requirement_is_in_the_stub(self):
        text = gen_stub.STUB.read_text(encoding="utf-8")
        self.assertIn("point_load_position is required when point_load > 0", text)

    def test_the_public_names_are_in_the_stub(self):
        text = gen_stub.STUB.read_text(encoding="utf-8")
        for name in civkit.__all__:
            self.assertRegex(text, rf"(?m)^(class |def ){name}\b|^{name}: ", name)


if __name__ == "__main__":
    unittest.main()
