# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-28 14:11 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)
"""CivKit: offline, traceable civil-engineering calculations.

    >>> import civkit
    >>> r = civkit.call("fluids.open_channel.manning_trapezoidal", bottom_width="1.5 m", side_slope=2.0,
    ...                 depth="0.6 m", manning_n=0.013, bed_slope=0.002)
    >>> r.outputs["discharge"].unit
    'm^3/s'

Every calculation runs in the Rust engine through its JSON contract; this package only reshapes inputs and results.
It has no dependencies and makes no network calls.

CivKit results support, and do not replace, the judgement of a qualified engineer.
"""

from . import _civkit, drawing
from ._api import call, describe
from ._api import list_calculations as list  # noqa: A001 (the API name of API_DESIGN §8)
from ._errors import CivKitError
from ._quantity import Q
from ._result import Result
from ._typed import build as _build

__version__: str = _civkit.version()

DISCLAIMER: str = _civkit.DISCLAIMER
"""Shown with every result (docs/ENGINEERING_STANDARDS.md §7); never part of the JSON."""


def version() -> str:
    """The CivKit engine version."""
    return _civkit.version()


__all__ = ["DISCLAIMER", "CivKitError", "Q", "Result", "call", "describe", "drawing", "list", "version"]

# The typed functions: civkit.<domain>.<topic>.<method>, one per installed calculation.
_namespaces = _build(list(), reserved=set(globals()))
globals().update(_namespaces)
__all__ += sorted(_namespaces)
del _namespaces
