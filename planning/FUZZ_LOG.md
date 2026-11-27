# DXF reader fuzzing log

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-27 18:02 IST · last edited 2026-11-27 21:59 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

Gate X6 of [ADR 0015](decisions/0015-dxf-reader.md) requires **8 hours of fuzzing in total before 0.1**, recorded here,
beside the 10-minute run in every Fuzz workflow run. A run counts when its **fuzzing step** completes green (no crash, no
heap-check failure); a crash is fixed (its input becomes a regression test) and the run is repeated. Other steps of
the workflow (the corpus cross-check) do not change the fuzzing time counted. Durations are the workflow's `max_total_time` for the run.

The routine runs (each push to `main`, and daily) give 10 minutes each, about 48 days to 8 hours on their own. To reach
the total sooner, run the Fuzz workflow by hand ("Run workflow") with a larger `seconds`; a job may last at most 6
hours, so for example two runs of 14 400 seconds (4 hours) each.

| Run | Date | Commit | Seconds | Result | Running total |
|---|---|---|---|---|---|
| [Fuzz #1](https://github.com/gowtham-sivarajan/civkit/actions/runs/45993081372) | 2026-11-27 | `241b44c` | 600 | green | 0 h 10 min |
| [Fuzz #2](https://github.com/gowtham-sivarajan/civkit/actions/runs/45997871210) | 2026-11-27 | `4e41e7a` | 600 | fuzzing green; run red at the corpus cross-check (a repeated-value disagreement, now refused: DXF_CROSS_CHECK_LOG RA-4) | 0 h 20 min |
| [Fuzz #3](https://github.com/gowtham-sivarajan/civkit/actions/runs/46002771589) | 2026-11-27 | `c740fd4` | 600 | green (fuzzing and the X2 report step); X2 summary: no disagreements | 0 h 30 min |
| [Fuzz #4](https://github.com/gowtham-sivarajan/civkit/actions/runs/46006337451) | 2026-11-27 | `ecc0193` | 600 | green (fuzzing and the X2 report step; summary not yet reviewed) | 0 h 40 min |
| [Fuzz #5](https://github.com/gowtham-sivarajan/civkit/actions/runs/46009067122) | 2026-11-27 | `435bb23` | 600 | green (fuzzing and the X2 report step; summary not yet reviewed) | 0 h 50 min |
| [Fuzz #6](https://github.com/gowtham-sivarajan/civkit/actions/runs/46018879706) | 2026-11-27 | `29d7230` | 600 | green (fuzzing and the X2 report step; summary not yet reviewed) | 1 h 0 min |
| [Fuzz #7](https://github.com/gowtham-sivarajan/civkit/actions/runs/46024301153) | 2026-11-27 | `07e200e` | 600 | green (fuzzing and the X2 report step; summary not yet reviewed) | 1 h 10 min |
| [Fuzz #8](https://github.com/gowtham-sivarajan/civkit/actions/runs/46028506621) | 2026-11-27 | `861f9fa` | 600 | green (fuzzing and the X2 report step; summary not yet reviewed) | 1 h 20 min |
| [Fuzz #9](https://github.com/gowtham-sivarajan/civkit/actions/runs/46030618706) | 2026-11-27 | `7fec8ed` | — | cancelled (superseded by the next push); not counted | 1 h 20 min |
| [Fuzz #10](https://github.com/gowtham-sivarajan/civkit/actions/runs/46032063323) | 2026-11-27 | `e9dfa91` | 600 | green (fuzzing and the X2 report step; summary not yet reviewed) | 1 h 30 min |

**Total so far: 1 h 30 min of 8 h.**
