# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-27 00:02 IST
"""call, list and describe over the Rust layer. Inputs are only reshaped into the JSON request; the engine does
every check (ADR 0014 point 4)."""

from __future__ import annotations

import json
from typing import Any, Mapping

from . import _civkit
from ._errors import CivKitError
from ._quantity import Q
from ._result import Result


def _run(request: dict[str, Any]) -> dict[str, Any]:
    """Sends one request through ``Registry::call_json``; returns the response or raises CivKitError."""
    response = json.loads(_civkit.call_json(json.dumps(request)))
    if "error" in response or "errors" in response:
        raise CivKitError.from_envelope(response)
    return response


def _number_or_raw(text: str) -> Any:
    try:
        return float(text)
    except ValueError:
        return text


def to_input(name: str, value: Any) -> dict[str, Any]:
    """One input in the contract's form ``{"value": ..., "unit": ...}``.

    Accepted: a dict (sent as is), ``Q``, ``(value, unit)``, ``"<number> <unit>"`` or a number as a string, a list
    of ``Q`` in one unit, a plain list, a number, or any other string (a choice, or a value the engine will judge).
    """
    if isinstance(value, Mapping):
        return dict(value)
    if isinstance(value, Q):
        return value.to_json()
    if isinstance(value, tuple) and len(value) == 2 and isinstance(value[1], str):
        v = value[0]
        return {"value": list(v) if isinstance(v, (list, tuple)) else v, "unit": value[1]}
    if isinstance(value, (list, tuple)) and value and all(isinstance(x, Q) for x in value):
        units = {(x.unit, x.negative) for x in value}
        if len(units) != 1:
            raise ValueError(
                f"{name}: a list input has one unit for the whole list, but these quantities use "
                f"{sorted(str(u) for u, _ in units)}; convert them to one unit first"
            )
        unit, negative = units.pop()
        return Q([x.value for x in value], unit, negative).to_json()
    if isinstance(value, (list, tuple)):
        return {"value": list(value)}
    if isinstance(value, str):
        text = value.strip()
        whole = _number_or_raw(text)
        if not isinstance(whole, str):
            return {"value": whole}
        head, _, unit = text.partition(" ")
        number = _number_or_raw(head)
        if not isinstance(number, str) and unit.strip():
            return {"value": number, "unit": unit.strip()}
        return {"value": value}
    return {"value": value}


def build_request(
    id: str, version: int | None, output_units: Mapping[str, str] | None, inputs: Mapping[str, Any]
) -> dict[str, Any]:
    """The JSON request for a call. ``None`` inputs are left out (an optional input not given)."""
    request: dict[str, Any] = {"id": id}
    if version is not None:
        request["version"] = version
    request["inputs"] = {k: to_input(k, v) for k, v in inputs.items() if v is not None}
    if output_units:
        request["output_units"] = dict(output_units)
    return request


def call(
    id: str, /, *, version: int | None = None, output_units: Mapping[str, str] | None = None, **inputs: Any
) -> Result:
    """Runs calculation ``id`` and returns its :class:`Result`, or raises :class:`CivKitError`.

    Each input may be ``"2.4 m^2"``, ``(2.4, "m^2")``, ``Q(2.4, "m^2")``, a bare number for a dimensionless input,
    a list (``([0, 90], "deg")`` or a list of ``Q`` in one unit), a choice string, or the contract's
    ``{"value": ..., "unit": ...}``. ``version`` defaults to the latest; ``output_units`` maps output names to units.
    """
    request = build_request(id, version, output_units, inputs)
    return Result(_run(request), request, _run)


def list_calculations() -> list[dict[str, Any]]:
    """Every registered calculation's descriptor."""
    return json.loads(_civkit.list_json())


def describe(id: str, version: int | None = None) -> dict[str, Any]:
    """One calculation's descriptor (the latest version unless ``version`` is given). Unknown: ``KeyError``."""
    text = _civkit.describe_json(id, version)
    if text is None:
        raise KeyError(id if version is None else f"{id} version {version}")
    return json.loads(text)
