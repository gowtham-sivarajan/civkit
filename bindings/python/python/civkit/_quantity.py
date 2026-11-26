# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-27 00:02 IST
"""Q: a value with a unit, as given to a calculation or returned by one."""

from __future__ import annotations

from typing import Any, Callable


class Q:
    """A quantity: ``Q(2.4, "m^2")``, a list ``Q([100, 50], "m")``, or a DMS angle ``Q([30, 15, 0], "dms")``.

    ``unit`` is ``None`` for a dimensionless value; ``negative`` marks a negative DMS angle. A quantity returned in
    a result can convert itself with :meth:`to`, which asks the engine for the result again in that unit.
    """

    __slots__ = ("value", "unit", "negative", "_convert")

    def __init__(self, value: Any, unit: str | None = None, negative: bool | None = None) -> None:
        self.value = list(value) if isinstance(value, tuple) else value
        self.unit = unit
        self.negative = negative
        self._convert: Callable[[str], Q] | None = None

    def to(self, unit: str) -> Q:
        """This result value in ``unit``, converted by the engine (the calculation is run again in that unit)."""
        if self._convert is None:
            raise ValueError(
                "only a quantity returned by a calculation can convert itself; CivKit converts units in the engine, "
                "so pass this quantity to a calculation and ask for the output unit you want"
            )
        return self._convert(unit)

    def to_json(self) -> dict[str, Any]:
        """The contract's input form: ``{"value": …, "unit": …}``."""
        d: dict[str, Any] = {"value": self.value}
        if self.unit is not None:
            d["unit"] = self.unit
        if self.negative is not None:
            d["negative"] = self.negative
        return d

    def __float__(self) -> float:
        return float(self.value)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Q):
            return NotImplemented
        return (self.value, self.unit, self.negative) == (other.value, other.unit, other.negative)

    def __hash__(self) -> int:
        v = tuple(self.value) if isinstance(self.value, list) else self.value
        return hash((v, self.unit, self.negative))

    def __repr__(self) -> str:
        extra = f", negative={self.negative!r}" if self.negative is not None else ""
        unit = f", {self.unit!r}" if self.unit is not None else ""
        return f"Q({self.value!r}{unit}{extra})"
