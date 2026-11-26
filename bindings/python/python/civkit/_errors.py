# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-27 00:02 IST
"""CivKitError: every contract error, as one exception class (ADR 0014 point 5)."""

from __future__ import annotations

import copy
from typing import Any


class CivKitError(Exception):
    """A calculation request failed.

    ``code``, ``input``, ``index`` and ``message`` are those of the first error; ``errors`` lists every error of the
    response, in order, as the engine sent them (dicts with ``code``, ``message`` and, where they apply, ``input``
    and ``index``). ``index`` is 0-based; the message names it 1-based ("leg 2").
    """

    def __init__(self, errors: list[dict[str, Any]]) -> None:
        if not errors:
            raise ValueError("CivKitError needs at least one error")
        self.errors: list[dict[str, Any]] = copy.deepcopy(errors)
        first = self.errors[0]
        self.code: str = first["code"]
        self.message: str = first.get("message", "")
        self.input: str | None = first.get("input")
        self.index: int | None = first.get("index")
        super().__init__(_describe(self.errors))

    @classmethod
    def from_envelope(cls, envelope: dict[str, Any]) -> CivKitError:
        """Builds the exception from ``{"error": {...}}`` or ``{"errors": [...]}``."""
        if "errors" in envelope:
            return cls(envelope["errors"])
        return cls([envelope["error"]])


def _one(e: dict[str, Any]) -> str:
    where = e.get("input")
    if where is not None and e.get("index") is not None:
        where = f"{where}[{e['index']}]"
    return f"{e['code']} ({where}): {e.get('message', '')}" if where else f"{e['code']}: {e.get('message', '')}"


def _describe(errors: list[dict[str, Any]]) -> str:
    if len(errors) == 1:
        return _one(errors[0])
    return f"{len(errors)} errors: " + "; ".join(_one(e) for e in errors)
