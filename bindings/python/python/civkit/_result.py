# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-28 14:11 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)
"""Result: a successful calculation response."""

from __future__ import annotations

import copy
import html
from typing import Any, Callable

from . import _civkit

from ._quantity import Q


class Result:
    """A successful calculation. ``to_dict()`` is the engine's JSON response exactly (gate P3).

    ``outputs`` maps each output name to a :class:`Q`, or to a list of ``Q`` for a list output. ``request`` is the
    JSON request that produced it. Its display (``repr``, and Jupyter's rich display) ends with the disclaimer;
    ``to_dict()`` does not carry it.

    CivKit results support, and do not replace, the judgement of a qualified engineer.
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
        return f"<Result {self.id} v{self.version}: {outs}>\n{_civkit.DISCLAIMER}"

    def _repr_html_(self) -> str:
        """Jupyter's rich display: the outputs as a table, then the disclaimer."""
        rows = []
        for name, q in self.outputs.items():
            values = q if isinstance(q, list) else [q]
            unit = values[0].unit if values else None
            shown = ", ".join(str(v.value) for v in values)
            cells = (html.escape(name), html.escape(shown), html.escape(unit or ""))
            rows.append("<tr>" + "".join(f"<td>{c}</td>" for c in cells) + "</tr>")
        return (
            f"<div><strong>{html.escape(self.id)}</strong> v{self.version} (CivKit {html.escape(self.civkit_version)})"
            "<table><thead><tr><th>output</th><th>value</th><th>unit</th></tr></thead>"
            f"<tbody>{''.join(rows)}</tbody></table>"
            f"<p><em>{html.escape(_civkit.DISCLAIMER)}</em></p></div>"
        )


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
