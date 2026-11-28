# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-28 14:12 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)
"""The Python API of API_DESIGN §8 and ADR 0014 (D-26): discovery, the input forms, results, typed functions."""

import inspect
import json
import unittest

import civkit
from civkit import _civkit

MANNING = "fluids.open_channel.manning_trapezoidal"
TRAVERSE = "surveying.traverse.compass_rule"
EARTHWORK = "quantities.earthwork.average_end_area"

MANNING_METRES = dict(
    bottom_width="1.5 m", side_slope=2.0, depth="0.6 m", manning_n=0.013, bed_slope=0.002
)


def raw(request: dict) -> dict:
    """The engine's own answer for a request, straight through the Rust layer."""
    return json.loads(_civkit.call_json(json.dumps(request)))


class TheRustLayerIsThin(unittest.TestCase):
    def test_exactly_six_functions(self):
        # ADR 0014's four calculation functions, and D-28's two drawing functions (ADR 0016 Q-A).
        # Besides them, one constant: the disclaimer (ENGINEERING_STANDARDS §7), so Python shows the engine's string.
        public = sorted(n for n in dir(_civkit) if not n.startswith("_"))
        functions = [n for n in public if callable(getattr(_civkit, n))]
        self.assertEqual(
            functions, ["call_json", "describe_json", "drawing_csv", "drawing_json", "list_json", "version"]
        )
        self.assertEqual(sorted(set(public) - set(functions)), ["DISCLAIMER"])
        self.assertIsInstance(_civkit.DISCLAIMER, str)

    def test_version_matches_the_engine(self):
        self.assertEqual(civkit.version(), _civkit.version())
        self.assertEqual(civkit.__version__, _civkit.version())


class Discovery(unittest.TestCase):
    def test_list_has_the_seven_calculations(self):
        ids = sorted({d["id"] for d in civkit.list()})
        self.assertEqual(
            ids,
            sorted([
                MANNING,
                "structural.beam.simply_supported",
                TRAVERSE,
                "surveying.levelling.height_of_instrument",
                "surveying.levelling.rise_and_fall",
                "surveying.levelling.closure_adjustment",
                EARTHWORK,
            ]),
        )

    def test_describe_returns_the_descriptor(self):
        d = civkit.describe(MANNING)
        self.assertEqual(d["id"], MANNING)
        self.assertEqual([i["name"] for i in d["inputs"]][0], "bottom_width")
        self.assertEqual(civkit.describe(MANNING, version=1)["version"], 1)

    def test_describe_unknown_is_a_key_error(self):
        with self.assertRaises(KeyError):
            civkit.describe("fluids.nothing.here")


class InputForms(unittest.TestCase):
    """The three forms of API_DESIGN §8 (and the JSON form) build the same request."""

    def test_string_tuple_quantity_and_json_forms_agree(self):
        string = civkit.call(MANNING, **MANNING_METRES).to_dict()
        tuple_ = civkit.call(
            MANNING, bottom_width=(1.5, "m"), side_slope=2.0, depth=(0.6, "m"),
            manning_n=0.013, bed_slope=0.002,
        ).to_dict()
        q = civkit.call(
            MANNING, bottom_width=civkit.Q(1.5, "m"), side_slope=2.0, depth=civkit.Q(0.6, "m"),
            manning_n=0.013, bed_slope=0.002,
        ).to_dict()
        as_json = civkit.call(
            MANNING, bottom_width={"value": 1.5, "unit": "m"}, side_slope={"value": 2.0},
            depth={"value": 0.6, "unit": "m"}, manning_n={"value": 0.013}, bed_slope={"value": 0.002},
        ).to_dict()
        self.assertEqual(string, tuple_)
        self.assertEqual(string, q)
        self.assertEqual(string, as_json)

    def test_a_bare_number_on_a_dimensional_input_is_missing_unit(self):
        with self.assertRaises(civkit.CivKitError) as e:
            civkit.call(MANNING, **{**MANNING_METRES, "depth": 0.6})
        self.assertEqual(e.exception.code, "MISSING_UNIT")
        self.assertEqual(e.exception.input, "depth")

    def test_a_list_input_as_numbers_with_one_unit(self):
        r = civkit.call(
            TRAVERSE,
            bearings=([0, 90, 180, 270], "deg"),
            distances=civkit.Q([100, 50, 99, 50], "m"),
        )
        self.assertEqual(len(r.outputs["latitudes"]), 4)

    def test_a_list_of_quantities_with_one_unit(self):
        a = civkit.call(TRAVERSE, bearings=([0, 90, 180, 270], "deg"), distances=([100, 50, 99, 50], "m"))
        b = civkit.call(
            TRAVERSE,
            bearings=[civkit.Q(x, "deg") for x in (0, 90, 180, 270)],
            distances=[civkit.Q(x, "m") for x in (100, 50, 99, 50)],
        )
        self.assertEqual(a.to_dict(), b.to_dict())

    def test_a_list_of_quantities_in_different_units_is_refused_before_the_call(self):
        with self.assertRaises(ValueError):
            civkit.call(TRAVERSE, bearings=([0, 90, 180], "deg"), distances=[civkit.Q(1, "m"), civkit.Q(1, "ft"), civkit.Q(1, "m")])

    def test_bearing_strings_go_as_a_tuple(self):
        r = civkit.call(
            TRAVERSE,
            bearings=(["N 30d E", "S 45d E", "N 75d W"], "bearing"),
            distances=([100, 200, 150], "ft"),
        )
        self.assertTrue(r.trace[0]["step"].startswith("leg 1: bearings: bearing"))

    def test_a_choice_is_a_plain_string(self):
        r = civkit.call(
            EARTHWORK, method="pyramid_at_zero_end",
            chainages=([0, 50], "m"), cut_areas=([0, 30], "m^2"), fill_areas=([0, 0], "m^2"),
        )
        self.assertEqual(r.trace[0], {"step": "method", "choice": "pyramid_at_zero_end"})
        self.assertAlmostEqual(r.outputs["cut_volumes"][0].value, 500.0)

    def test_output_units(self):
        r = civkit.call(MANNING, output_units={"velocity": "ft/s"}, **MANNING_METRES)
        self.assertEqual(r.outputs["velocity"].unit, "ft/s")


class Results(unittest.TestCase):
    def test_outputs_are_quantities_and_to_dict_is_the_contract_json(self):
        r = civkit.call(MANNING, **MANNING_METRES)
        q = r.outputs["discharge"]
        self.assertIsInstance(q, civkit.Q)
        self.assertEqual(q.unit, "m^3/s")
        self.assertIsInstance(q.value, float)
        request = r.request
        self.assertEqual(r.to_dict(), raw(request))
        self.assertEqual(r.id, MANNING)
        self.assertEqual(r.version, 1)
        self.assertEqual(r.trace, r.to_dict()["trace"])
        self.assertEqual(r.warnings, [])
        self.assertTrue(r.references and r.assumptions)

    def test_to_dict_is_a_copy(self):
        r = civkit.call(MANNING, **MANNING_METRES)
        r.to_dict()["outputs"].clear()
        self.assertIn("discharge", r.to_dict()["outputs"])

    def test_list_outputs_are_lists_of_quantities(self):
        r = civkit.call(TRAVERSE, bearings=([0, 90, 180, 270], "deg"), distances=([100, 50, 99, 50], "m"))
        lat = r.outputs["latitudes"]
        self.assertIsInstance(lat, list)
        self.assertTrue(all(isinstance(x, civkit.Q) and x.unit == "m" for x in lat))

    def test_to_converts_through_the_engine(self):
        r = civkit.call(MANNING, **MANNING_METRES)
        v = r.outputs["velocity"]
        in_feet = v.to("ft/s")
        self.assertEqual(in_feet.unit, "ft/s")
        self.assertAlmostEqual(in_feet.value, v.value / 0.3048, places=9)
        # A list output converts element by element, each keeping its own position.
        t = civkit.call(TRAVERSE, bearings=([0, 90, 180, 270], "deg"), distances=([100, 50, 99, 50], "m"))
        for i, lat in enumerate(t.outputs["latitudes"]):
            self.assertAlmostEqual(lat.to("ft").value, lat.value / 0.3048, places=9, msg=f"leg {i + 1}")
        # A converted value converts again, from its new request.
        self.assertAlmostEqual(in_feet.to("m/s").value, v.value, places=12)

    def test_warnings_come_from_the_response(self):
        # No built-in calculation warns yet (validity rules need a cited basis, D-24), so a real response is given a
        # warning here to show that Result keeps it as sent.
        request = civkit.call(MANNING, **MANNING_METRES).request
        response = raw(request)
        warning = {"code": "OUTSIDE_VALIDITY_RANGE", "level": "caution", "message": "example", "input": "depth"}
        response["warnings"] = [warning]
        r = civkit.Result(response, request, raw)
        self.assertEqual(r.warnings, [warning])
        self.assertEqual(r.to_dict(), response)

    def test_the_request_is_plain_json(self):
        r = civkit.call(TRAVERSE, bearings=((0, 90, 180, 270), "deg"), distances=civkit.Q((100, 50, 99, 50), "m"))
        self.assertEqual(r.request["inputs"]["bearings"], {"value": [0, 90, 180, 270], "unit": "deg"})
        self.assertEqual(r.request["inputs"]["distances"], {"value": [100, 50, 99, 50], "unit": "m"})
        self.assertEqual(json.loads(json.dumps(r.request)), r.request)

    def test_to_a_unit_of_the_wrong_dimension_raises(self):
        r = civkit.call(MANNING, **MANNING_METRES)
        with self.assertRaises(civkit.CivKitError) as e:
            r.outputs["velocity"].to("m")
        self.assertEqual(e.exception.code, "UNIT_MISMATCH")

    def test_a_hand_made_quantity_cannot_convert(self):
        with self.assertRaises(ValueError):
            civkit.Q(1.0, "m").to("ft")


class TypedFunctions(unittest.TestCase):
    def test_the_typed_function_equals_the_generic_call(self):
        typed = civkit.fluids.open_channel.manning_trapezoidal(**MANNING_METRES)
        generic = civkit.call(MANNING, version=1, **MANNING_METRES)
        self.assertEqual(typed.to_dict(), generic.to_dict())

    def test_every_registered_calculation_has_a_typed_function(self):
        for d in civkit.list():
            domain, topic, method = d["id"].split(".")
            fn = getattr(getattr(getattr(civkit, domain), topic), method)
            self.assertTrue(callable(fn), d["id"])

    def test_the_signature_is_keyword_only_with_the_descriptor_inputs(self):
        fn = civkit.surveying.traverse.compass_rule
        params = inspect.signature(fn).parameters
        self.assertEqual(list(params), ["bearings", "distances", "output_units"])
        self.assertIs(params["bearings"].default, inspect.Parameter.empty)
        self.assertIsNone(params["output_units"].default)
        self.assertTrue(all(p.kind is inspect.Parameter.KEYWORD_ONLY for p in params.values()))

    def test_the_docstring_is_built_from_the_descriptor(self):
        doc = civkit.fluids.open_channel.manning_trapezoidal.__doc__
        d = civkit.describe(MANNING)
        self.assertIn(d["title"], doc)
        self.assertIn("bottom_width", doc)
        self.assertIn("Validity range: none is stated", doc)

    def test_the_docstring_writes_dimensions_as_the_engine_does(self):
        doc = civkit.fluids.open_channel.manning_trapezoidal.__doc__
        self.assertIn("bottom_width [L, e.g. m]", doc)
        self.assertIn("discharge [L^3*T^-1, e.g. m^3/s]", doc)
        self.assertIn("side_slope [dimensionless]", doc)
        self.assertNotIn("'length'", doc)

    def test_a_conditional_requirement_is_prominent(self):
        """IMPLEMENTATION_PLAN Phase 3 task 5: conditions stay in prose (D-23 §3.9), but are shown prominently."""
        doc = civkit.structural.beam.simply_supported.__doc__
        self.assertIn("point_load_position [L, e.g. m, optional; REQUIRED WHEN point_load > 0]", doc)
        self.assertIn("Conditional requirements:\n    - point_load_position is required when point_load > 0", doc)
        manning = civkit.fluids.open_channel.manning_trapezoidal.__doc__
        self.assertNotIn("Conditional requirements", manning)

    def test_the_typed_function_sends_its_descriptor_version(self):
        r = civkit.structural.beam.simply_supported(
            span="4 m", elastic_modulus="200 GPa", second_moment="0.0001 m^4", position="2 m", udl="1 kN/m"
        )
        self.assertEqual(r.request["version"], 1)


if __name__ == "__main__":
    unittest.main()
