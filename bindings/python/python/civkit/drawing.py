# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 22:52 IST · last edited 2026-11-28 14:11 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)
"""Drawing analysis (D-28, ADR 0016; API_DESIGN §13): read-only quantities of a DXF drawing.

    >>> import civkit
    >>> report = civkit.drawing.analyze("site.dxf")          # doctest: +SKIP
    >>> csv_text = civkit.drawing.schedule_csv("site.dxf")   # doctest: +SKIP

Both run in the Rust engine (`civkit_drawing::analyze_dxf`), which also validates `detail`; this module only reads
the file and reshapes errors (ADR 0014 point 4: no validation of its own).
Nothing is written: save the schedule yourself if you want it on disk. Strings in the report that come from the
drawing (layer and block names, text values) are data.

CivKit results support, and do not replace, the judgement of a qualified engineer.
"""

from __future__ import annotations

import html
import json
import os
from typing import Any, Literal

from . import _civkit
from ._errors import CivKitError

Source = str | os.PathLike[str] | bytes | bytearray | memoryview
"""A path to a DXF file, or its bytes."""


def _bytes(source: Source) -> bytes:
    if isinstance(source, (bytes, bytearray, memoryview)):
        return bytes(source)
    if isinstance(source, (str, os.PathLike)):
        with open(source, "rb") as f:
            return f.read()
    raise TypeError(f"a path or bytes is needed, not {type(source).__name__}")


def _result(ok: bool, text: str) -> str:
    if not ok:
        raise CivKitError.from_envelope(json.loads(text))
    return text


class Report(dict):
    """The drawing report: exactly the engine's JSON as a dict (equal to it, and `json.dumps` gives the same data).
    Only its display adds the disclaimer: `repr`, and Jupyter's rich display, which escapes every string from the
    drawing.
    """

    def __repr__(self) -> str:
        return f"{dict.__repr__(self)}\n{_civkit.DISCLAIMER}"

    def _repr_html_(self) -> str:
        units = self.get("drawing", {}).get("units", {})
        si = units.get("stated") and units.get("to_metres") is not None
        rows = []
        for layer in self.get("layers", []):
            q = (layer.get("si") if si else layer.get("drawing_units")) or {}
            count = sum(layer.get("counts", {}).values())
            flag = " (incomplete)" if layer.get("incomplete") else ""
            area = sum(v for k, v in q.items() if "area" in k and "hatch" not in k)
            length = sum(v for k, v in q.items() if "length" in k or "perimeter" in k or "circumference" in k)
            rows.append(f"<tr><td>{html.escape(str(layer.get('name')))}{flag}</td><td>{count}</td>"
                        f"<td>{length:.6g}</td><td>{area:.6g}</td></tr>")
        unit = "m" if si else "drawing units"
        cautions = sum(1 for w in self.get("warnings", []) if w.get("level") == "caution")
        return (
            f"<div><strong>drawing.analyze</strong> {html.escape(str(self.get('version')))} · units: "
            f"{html.escape(str(units.get('name') or 'not stated'))} · {cautions} caution(s)"
            f"<table><thead><tr><th>layer</th><th>entities</th><th>length + perimeter ({unit})</th>"
            f"<th>area ({unit}²)</th></tr></thead><tbody>{''.join(rows)}</tbody></table>"
            f"<p><em>{html.escape(_civkit.DISCLAIMER)}</em></p></div>"
        )


def analyze(source: Source, *, detail: Literal["layer", "entity"] = "layer") -> Report:
    """The drawing report (`drawing.analyze`, API_DESIGN §13) as a dict (a `Report`, whose display adds the
    disclaimer). `detail="entity"` adds one row per placed, measured entity.

    Raises `CivKitError` (`DRAWING_UNREADABLE`, `DRAWING_LIMIT`, or `INVALID_VALUE` for `detail`), and the usual
    `OSError` when the file cannot be opened.

    CivKit results support, and do not replace, the judgement of a qualified engineer.
    """
    return Report(json.loads(_result(*_civkit.drawing_json(_bytes(source), detail))))


def schedule_csv(source: Source) -> str:
    """The quantity schedule as CSV text (RFC 4180, CRLF), exactly as the engine writes it. Layer names that
    spreadsheets would read as formulas are prefixed with `'`.

    Raises `CivKitError` (`DRAWING_UNREADABLE`, `DRAWING_LIMIT`), and `OSError` when the file cannot be opened.

    CivKit results support, and do not replace, the judgement of a qualified engineer.
    """
    return _result(*_civkit.drawing_csv(_bytes(source)))


__all__ = ["Report", "Source", "analyze", "schedule_csv"]
