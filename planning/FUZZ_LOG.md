# DXF reader fuzzing log

> **Provenance:** Claude Sonnet 5.5 (`claude-sonnet-5-5`) · first drafted 2026-11-27 18:02 IST · last edited 2026-11-27 18:10 IST by Claude Sonnet 5.5 (`claude-sonnet-5-5`)

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
| [Fuzz #2](https://github.com/gowtham-sivarajan/civkit/actions/runs/45997871210) | 2026-11-27 | `4e41e7a` | 600 | fuzzing green; run red at the corpus cross-check (see AI_HANDOVER) | 0 h 20 min |

**Total so far: 0 h 20 min of 8 h.**
