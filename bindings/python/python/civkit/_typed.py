# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-28 14:11 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)
"""Typed functions, generated at import from the installed descriptors (ADR 0014 point 6):
``civkit.fluids.open_channel.manning_trapezoidal(bottom_width="1.5 m", ...)``."""

from __future__ import annotations

from . import _civkit  # the disclaimer: the engine's one string

import inspect
from typing import Any, Callable

from ._api import call
from ._result import Result


class Namespace:
    """A level of the calculation ID (``civkit.fluids``, ``civkit.fluids.open_channel``)."""

    def __init__(self, path: str) -> None:
        self._path = path

    def __repr__(self) -> str:
        return f"<civkit namespace {self._path}>"


_DIMENSION_SYMBOLS = [
    ("length", "L"), ("mass", "M"), ("time", "T"), ("temperature", "Θ"), ("amount", "N"), ("current", "I"),
    ("luminous_intensity", "J"), ("angle", "angle"),
]

# A conditional requirement stays in prose in 0.1 (D-23 §3.9), written as "...; required when <condition>" at the end of
# an optional input's description. Interfaces show it prominently (IMPLEMENTATION_PLAN Phase 3 task 5).
_CONDITION_MARK = "required when "


def dimension_text(dimension: dict[str, int]) -> str:
    """A dimension as the engine writes it in messages: `L`, `L^3*T^-1`, `1` for dimensionless."""
    parts = []
    for key, symbol in _DIMENSION_SYMBOLS:
        exponent = dimension.get(key, 0)
        if exponent:
            parts.append(symbol if exponent == 1 else f"{symbol}^{exponent}")
    return "*".join(parts) or "1"


def condition(spec: dict[str, Any]) -> str | None:
    """For an optional input that is required under a condition, the condition ("point_load > 0")."""
    if spec.get("required", True):
        return None
    text = spec.get("description", "")
    at = text.lower().rfind(_CONDITION_MARK)
    if at < 0:
        return None
    return text[at + len(_CONDITION_MARK):].strip().rstrip(".")


def unit_text(spec: dict[str, Any]) -> str:
    """How an input or output is given: its dimension and a unit, a list, or the allowed choices."""
    if spec.get("kind") == "choice":
        return "one of: " + ", ".join(spec["choices"])
    unit = spec.get("display_unit") or spec.get("si_unit")
    text = f"{dimension_text(spec['dimension'])}, e.g. {unit}" if unit else "dimensionless"
    if spec.get("kind") == "list":
        text = f"list, {text}"
    if spec.get("accepts_bearing"):
        text += "; or quadrant bearings with unit 'bearing'"
    return text


def requirement_text(spec: dict[str, Any]) -> str:
    """'' for a required input, ', optional', or ', optional; REQUIRED WHEN <condition>'."""
    if spec["required"]:
        return ""
    when = condition(spec)
    return f", optional; REQUIRED WHEN {when}" if when else ", optional"


def docstring(d: dict[str, Any]) -> str:
    """The typed function's docstring, from the descriptor."""
    lines = [d["title"], "", d["summary"], "", "Inputs (keyword-only):"]
    for i in d["inputs"]:
        lines.append(f"    {i['name']} [{unit_text(i)}{requirement_text(i)}]: {i['description']}")
    conditional = [(i["name"], condition(i)) for i in d["inputs"] if condition(i)]
    if conditional:
        lines += ["", "Conditional requirements:"] + [f"    - {n} is required when {c}" for n, c in conditional]
    lines += ["", "Outputs:"]
    for o in d["outputs"]:
        lines.append(f"    {o['name']} [{unit_text(o)}]: {o['description']}")
    lines += ["", "Assumptions:"] + [f"    - {a}" for a in d["assumptions"]]
    if d.get("validity"):
        lines += ["", "Validity rules:"] + [f"    - {v}" for v in d["validity"]]
    lines += ["", "References:"] + [f"    - {r}" for r in d["references"]]
    lines += ["", f"Calculation {d['id']}, version {d['version']}. Returns a civkit.Result; raises civkit.CivKitError."]
    lines += ["", _civkit.DISCLAIMER]
    return "\n".join(lines)


def make_function(d: dict[str, Any]) -> Callable[..., Result]:
    """One typed function. It sends the descriptor version it was generated from."""
    calc_id, version = d["id"], d["version"]

    def function(*, output_units: dict[str, str] | None = None, **inputs: Any) -> Result:
        return call(calc_id, version=version, output_units=output_units, **inputs)

    params = [
        inspect.Parameter(
            i["name"], inspect.Parameter.KEYWORD_ONLY, default=inspect.Parameter.empty if i["required"] else None
        )
        for i in d["inputs"]
    ]
    params.append(inspect.Parameter("output_units", inspect.Parameter.KEYWORD_ONLY, default=None))
    function.__signature__ = inspect.Signature(params, return_annotation=Result)  # type: ignore[attr-defined]
    function.__name__ = calc_id.rsplit(".", 1)[-1]
    function.__qualname__ = calc_id
    function.__module__ = "civkit"
    function.__doc__ = docstring(d)
    return function


def build(descriptors: list[dict[str, Any]], reserved: set[str]) -> dict[str, Namespace]:
    """The top-level namespaces (``fluids``, ``surveying`` ...) holding a function per calculation ID."""
    roots: dict[str, Namespace] = {}
    for d in sorted(descriptors, key=lambda d: d["id"]):
        *path, method = d["id"].split(".")
        if not path or path[0] in reserved:
            continue  # cannot be placed without hiding part of the API; still reachable through civkit.call
        node = roots.setdefault(path[0], Namespace(path[0]))
        for depth, part in enumerate(path[1:], start=2):
            if not hasattr(node, part):
                setattr(node, part, Namespace(".".join(path[:depth])))
            node = getattr(node, part)
        setattr(node, method, make_function(d))
    return roots
