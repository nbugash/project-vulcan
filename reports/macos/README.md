# Measurements from the authoritative runner

Gate 5 calls the Apple Silicon runner authoritative because that hardware
reproduces the baseline's asymmetric cores. Until 2026-09-19 that had never been
checked on a machine that has any.

## 2026-09-19 — Apple M3 Pro

| | |
|---|---|
| Chip | Apple M3 Pro, 5 performance + 6 efficiency cores |
| Memory | 18 GB |
| System | macOS 26.6.2, rendering through Metal |
| Baseline | 6 cores as 2P + 4E, 8 GB |

The runner is a larger machine than the baseline, so these figures read
optimistic. Each measurement records the topology it was taken on, which is why
that is visible here rather than hidden inside the number.

| Metric | Measured | Budget | |
|---|---|---|---|
| Keystroke to paint, p99 | 0.59 ms | 8 ms | pass |
| Longest UI-thread task | 4.66 ms | 8 ms | pass |
| Cold start to a rendered file | 109.87 ms | 300 ms | pass |
| Peak resident memory | 86.34 MB | 2.5 GB | pass |
| Typical resident memory | 86.34 MB | 1.5 GB | pass |
| Idle resident memory | 86.34 MB | 400 MB | pass |
| Idle processor, mean | 1.5 % | 2 % | pass, after the amendment below |

The gate returned a verdict rather than a refusal, which is what T102 asked for.

Idle processor exceeded its 1% budget, and T119 investigated rather than adjusting
the number. The shell draws **zero frames** across an idle window, and an empty
GPUI window containing one element idles at 1.34%, 1.19% and 1.10% — the same as
the whole shell. The cost is the framework's event loop and nothing this product
does. Constitution v4.1.0 sets the budget to 2% on that evidence: above the
framework's floor, below anything that would stop the check working.

## What the five runs cost, and bought

Five runs on this machine found five defects that no Linux run could have, and
four of them were in the measurement rather than the product:

1. Memory was never measured on macOS: the reader was `/proc/self/statm`.
2. Idle processor time was inferred from how far a sleep overran, which measures
   the platform's timer granularity. It reported 10% for a window doing nothing.
3. Both readers then shelled out to `ps`, and sampling runs on every
   observation, so a three hundred sample run forked several hundred times and
   was charged for it. A sleeping thread measured two thirds of a core.
4. Budgets stated as p99 were reported as the maximum of the run, over a sample
   too small for a percentile to differ from the maximum. Two runs minutes apart
   reported 3.97 ms and 16.88 ms for the same work.
5. The latency profile was applied and then refused, because `dnctl` prints a
   delay as `10 ms` and the parser expected `40ms`.

Only the last was a straightforward bug. The others were measurements that
looked plausible and were not, which is the failure mode this gate exists to
prevent and was itself committing.
