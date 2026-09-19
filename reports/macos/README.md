# Measurements from the authoritative runner

Gate 5 calls the Apple Silicon runner authoritative because that hardware
reproduces the baseline's asymmetric cores. Until 2026-09-19 that had never been
checked on a machine that has any.

## 2026-09-19 — Apple M3 Pro, all gates pass

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
| Keystroke to paint, p99 | 0.57 ms | 8 ms | pass |
| Longest UI-thread task | 5.27 ms | 8 ms | pass |
| Cold start to a rendered file | 95.24 ms | 300 ms | pass |
| Peak resident memory | 86.67 MB | 2.5 GB | pass |
| Typical resident memory | 86.67 MB | 1.5 GB | pass |
| Idle resident memory | 86.67 MB | 400 MB | pass |
| Idle processor, mean | 1.57 % | 2 % | pass |

`gate-budget` returns `passed` on the authoritative runner at both the 0 ms and
10 ms round-trip profiles, with the latency profile applied through `dnctl` and
read back. Every one of the nine checks passes.

### The idle budget

The 1% the constitution originally carried was set before anything had been
measured. An empty GPUI window containing one element idles at 1.34%, 1.19% and
1.10%, which is the same as the whole shell, and the shell draws zero frames
across an idle window. The floor belongs to the framework's event loop, not to
anything this product does, so constitution v4.1.0 sets the budget at 2%:
above the floor, and still failing if what Vulcan adds on top of it doubles.

## What the seven runs cost, and bought

Seven runs on this machine found six defects that no Linux run could have, and
five of them were in the measurement rather than the product:

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

6. The idle window began the instant a three hundred input burst ended, so the
   frames still queued behind it drained inside the window and were charged to
   idling. One run read 15.32% where quiet runs read 1.5%.

Only the fifth was a straightforward bug. The others were measurements that
looked plausible and were not, which is the failure mode this gate exists to
prevent and was itself committing.
