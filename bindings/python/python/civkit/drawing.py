# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 22:52 IST · last edited 2026-11-27 22:53 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)
"""Drawing analysis (D-28, ADR 0016; API_DESIGN §13): read-only quantities of a DXF drawing.

    >>> import civkit
    >>> report = civkit.drawing.analyze("site.dxf")          # doctest: +SKIP
    >>> csv_text = civkit.drawing.schedule_csv("site.dxf")   # doctest: +SKIP

Both run in the Rust engine (`civkit_drawing::analyze_dxf`), which also validates `detail`; this module only reads
the file and reshapes errors (ADR 0014 point 4: no validation of its own).
Nothing is written: save the schedule yourself if you want it on disk. Strings in the report that come from the
drawing (layer and block names, text values) are data.
"""

from __future__ import annotations

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


def analyze(source: Source, *, detail: Literal["layer", "entity"] = "layer") -> dict[str, Any]:
    """The drawing report (`drawing.analyze`, API_DESIGN §13) as a dict. `detail="entity"` adds one row per placed,
    measured entity.

    Raises `CivKitError` (`DRAWING_UNREADABLE`, `DRAWING_LIMIT`, or `INVALID_VALUE` for `detail`), and the usual
    `OSError` when the file cannot be opened.
    """
    return json.loads(_result(*_civkit.drawing_json(_bytes(source), detail)))


def schedule_csv(source: Source) -> str:
    """The quantity schedule as CSV text (RFC 4180, CRLF), exactly as the engine writes it. Layer names that
    spreadsheets would read as formulas are prefixed with `'`.

    Raises `CivKitError` (`DRAWING_UNREADABLE`, `DRAWING_LIMIT`), and `OSError` when the file cannot be opened.
    """
    return _result(*_civkit.drawing_csv(_bytes(source)))


__all__ = ["Source", "analyze", "schedule_csv"]
