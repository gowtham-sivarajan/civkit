# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-27 00:02 IST
"""Result: a successful calculation response."""

from __future__ import annotations

import copy
from typing import Any, Callable

from ._quantity import Q


class Result:
    """A successful calculation. ``to_dict()`` is the engine's JSON response exactly (gate P3).

    ``outputs`` maps each output name to a :class:`Q`, or to a list of ``Q`` for a list output. ``request`` is the
    JSON request that produced it.
    """

    def __init__(self, response: dict[str, Any], request: dict[str, Any], rerun: Callable[[dict], dict]) -> None:
        self._response = copy.deepcopy(response)
        self.request: dict[str, Any] = copy.deepcopy(request)
        self.id: str = response["id"]
        self.version: int = response["version"]
        self.civkit_version: str = response["civkit_version"]
        self.trace: list[dict[str, Any]] = copy.deepcopy(response["trace"])
        self.warnings: list[dict[str, Any]] = copy.deepcopy(response["warnings"])
        self.references: list[str] = list(response["references"])
        self.assumptions: list[str] = list(response["assumptions"])
        self.outputs: dict[str, Q | list[Q]] = {}
        for name, out in response["outputs"].items():
            value, unit = out["value"], out.get("unit")
            if isinstance(value, list):
                self.outputs[name] = [_bound(v, unit, request, name, i, rerun) for i, v in enumerate(value)]
            else:
                self.outputs[name] = _bound(value, unit, request, name, None, rerun)

    def to_dict(self) -> dict[str, Any]:
        """The contract JSON response, as a fresh dict."""
        return copy.deepcopy(self._response)

    def __repr__(self) -> str:
        outs = ", ".join(f"{k}={v!r}" for k, v in self.outputs.items())
        return f"<Result {self.id} v{self.version}: {outs}>"


def _bound(value: Any, unit: str | None, request: dict, name: str, index: int | None, rerun: Callable) -> Q:
    """A result quantity that converts itself by re-running its request with ``output_units`` set."""
    q = Q(value, unit)

    def convert(new_unit: str) -> Q:
        again = copy.deepcopy(request)
        again["output_units"] = {**again.get("output_units", {}), name: new_unit}
        out = rerun(again)["outputs"][name]
        v = out["value"] if index is None else out["value"][index]
        return _bound(v, out.get("unit"), again, name, index, rerun)

    q._convert = convert
    return q
