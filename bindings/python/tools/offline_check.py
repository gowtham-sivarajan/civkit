# Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:02 IST · last edited 2026-11-27 00:02 IST
"""Gate P1 (ADR 0014): after an offline wheel install, run one list calculation and one contract error with every
socket call refused. Run with the interpreter the wheel was installed into; prints "P1 ok" or exits non-zero.

On Linux CI also runs it inside a network namespace with no interfaces; elsewhere the refused sockets and pip's
--no-index are the offline guarantee.
"""

import socket
import sys


def refuse(*args, **kwargs):
    raise RuntimeError("network access attempted")


socket.socket = refuse
socket.create_connection = refuse
socket.getaddrinfo = refuse

import civkit  # noqa: E402 (after the sockets are blocked, on purpose)

r = civkit.surveying.traverse.compass_rule(bearings=([0, 90, 180, 270], "deg"), distances=([100, 50, 99, 50], "m"))
assert len(r.outputs["adjusted_latitudes"]) == 4, r
try:
    civkit.call("fluids.open_channel.manning_trapezoidal", bottom_width="1.5 m", side_slope=2.0, depth=0.6,
                manning_n=0.013, bed_slope=0.002)
except civkit.CivKitError as e:
    assert (e.code, e.input) == ("MISSING_UNIT", "depth"), e
else:
    sys.exit("expected MISSING_UNIT for a bare depth")
print(f"P1 ok: civkit {civkit.__version__} on Python {sys.version.split()[0]}")
