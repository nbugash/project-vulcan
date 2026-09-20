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

`gate-budget` returns `passed` on the authoritative runner at both the 0 ms and
10 ms round-trip profiles, with the latency profile applied through `dnctl` and
read back. Every one of the nine checks passes.

### The idle budget, and why it is gone

It read 1.57% here against a 2% budget. Constitution v4.2.0 removed the budget
rather than keeping the pass.

The shell draws zero frames while idle, and an empty window of the framework
costs the same as the whole shell, so the figure was the framework's floor
rather than anything this product does. More to the point, the product does not
exist yet: no plugin host, no language server, no indexer, no terminal. Every
one of those changes the idle profile and several will dominate it.

The measurement was also machine-dependent in a way that mattered. It is
processor time over wall time, so identical work reads higher on a slower core,
and this runner is two to four times faster than the baseline. The number would
have meant something different on the machine the budget describes.

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
