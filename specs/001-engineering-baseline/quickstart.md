# Quickstart: Engineering Baseline

**Branch**: `001-engineering-baseline` | **Date**: 2026-09-18 | **Plan**: [plan.md](./plan.md)

How to run every gate and confirm it does what the specification says. Each scenario below is
a check that the gate fails when it should, not only that it passes when nothing is wrong. A
gate that has never been seen to fail has not been validated.

## Prerequisites

| Requirement | Why |
| --- | --- |
| Rust toolchain, version from `rust-toolchain.toml` | Gates 1, 5 and 8 are workspace members. |
| Python 3.11+ | Gate 9, the pre-existing `featuremap` extension. |
| Container runtime | The pinned comparison environment for gate 8, which runs a headless `sway` session inside it. |
| `tc` with `netem` | Round-trip simulation for gate 5. Linux, or inside the container. |

Vendored assets are already in the repository: `mockups/fonts/` and `mockups/icons/`. No
network access is required for any gate to render correctly.

## Gate 1 — architectural boundaries

```bash
cargo run -p gate-boundary
```

Expected: exit `0`, no findings.

**Prove it fails.** Add a dependency on `vulcan-adapters` to `crates/vulcan-domain/Cargo.toml`,
then re-run. Expected: exit `2`, naming both crates and the manifest line. Remove it and
confirm exit `0` returns.

The compiler catches most of this class already; this gate exists for the edge the compiler
accepts, which is a dependency edge pointing the wrong way through the workspace.

## Gate 5 — resource budgets

```bash
cargo run -p gate-budget -- --runner linux-cgroup --rtt 0
```

Expected: one measurement per metric, each with a verdict, and the report naming the observed
core topology.

**Prove it refuses.** Run outside the constrained environment. Expected: exit `1` with a
message that constraints could not be enforced, and no measurements written. This is the
behaviour FR-005 requires: refusing is not the same as failing, and neither is the same as
passing.

**Prove it fails a budget.** Build with an artificial delay on the frame path, re-run.
Expected: exit `2`, naming the metric, its budget and the measured value.

**Full sweep**, as the build runs it:

```bash
cargo run -p gate-budget -- --runner apple-silicon
```

Runs every round-trip profile. Only the Apple Silicon result is authoritative; the Linux run
is advisory and recorded alongside it.

## Gate 8 — prototype fidelity

```bash
cargo run -p gate-fidelity -- extract
cargo run -p gate-fidelity -- lint
```

Expected from `extract`: a token set covering the stylesheet's custom properties and the
fourteen values documented only in `mockups/assets.md`. Run it twice and confirm the output is
byte-identical.

Expected from `lint`: exit `0` on clean source.

**Prove the lint fails.** Introduce a literal colour in the reproduction source, re-run.
Expected: exit `2`, naming the value, the file and the line.

**Comparison**, which must run inside the pinned environment:

```bash
./tools/gate-fidelity/run-in-pinned-env.sh compare
```

Expected: exit `0`, no pixel difference.

**Prove the comparison fails.** Change one spacing value in the reproduction by a single
pixel, re-run. Expected: exit `2`, with a visual report showing what moved. This is SC-005.

**Prove it refuses outside the environment.** Run `compare` directly on the host. Expected:
exit `1`, not a passing or failing comparison.

**Why a compositor rather than a virtual framebuffer.** GPUI presents a Vulkan swapchain. On
X11 that handoff needs DRI3, which a machine with no GPU does not have, so capture under Xvfb
or Xorg's dummy driver returns a black image with no error to explain it. Wayland presents
through shared memory instead, and wlroots' headless backend supplies the virtual seat GPUI
requires. If a capture ever comes back uniformly black, check that the session is Wayland
before looking anywhere else.

## The shell

```bash
cargo run -p shell-preview
```

Expected: the approved interface, at the prototype's viewport, with every region present.

**Prove it matches.** Run the comparison inside the pinned environment and confirm no pixel
differs:

```bash
./tools/gate-fidelity/run-in-pinned-env.sh compare
```

**Prove it responds.** With the shell running, in order:

| Action | Expected |
| --- | --- |
| Drag a tool window edge | Resizes within the frame budget, staying within the declared widths |
| Click a tool window's collapse control | Collapses and restores |
| Switch an editor tab | Switches within the frame budget |
| Open the command palette and type | Appears, filters, dismisses |
| Change `density` between compact, default and roomy | Every dependent dimension changes to the manifest's value |
| Change `toolSide` | Tool windows move to the other side |
| Activate any toolbar control | Shows its active treatment and does nothing else |

That last row is the contract, not a limitation. A control that partly works here would be
indistinguishable from a defect once a later feature owns it.

**Prove the budgets hold against something real:**

```bash
cargo run -p gate-budget -- --runner apple-silicon
```

Expected: every metric measured against the running shell rather than a placeholder. This is
why the shell is in this feature.

**Capture it for review.** The last thing a feature does is leave something a person can
look at without building the branch:

```bash
cargo run -p shell-preview -- --screenshot reports/screenshots/F000-engineering-baseline.png
```

Expected: a lossless capture at the prototype's viewport. Repeat per notable state, suffixing
the name, at minimum the palette open and each density profile. These are for human review; the
pixel comparison in gate 8 is the machine's check and neither replaces the other.

**Prove undepicted states are caught.** Resize the window away from the prototype's viewport
with no recorded prototype extension for it. Expected: the build fails, naming the unrecorded
state.

## Gate 9 — feature sequencing

```bash
python3 .specify/extensions/featuremap/scripts/python/feature_map.py verify
python3 .specify/extensions/featuremap/scripts/python/feature_map.py resolve
```

Expected: a consistent map, and the next feature whose dependencies are satisfied.

**Prove it blocks.** Request a feature whose dependencies are unfinished:

```bash
python3 .specify/extensions/featuremap/scripts/python/feature_map.py resolve F001
```

Expected: exit `2`, naming F000 as the blocker.

Already delivered. This scenario confirms it, rather than validating new work.

## The suite as the build runs it

```bash
cargo test --workspace
cargo run -p shell-preview --  --smoke
cargo run -p gate-boundary
cargo run -p gate-budget -- --runner apple-silicon
./tools/gate-fidelity/run-in-pinned-env.sh compare
python3 .specify/extensions/featuremap/scripts/python/test_feature_map.py
```

Expected total: under fifteen minutes (SC-007). If it exceeds that, the suite is the problem,
because a suite people avoid protects nothing.

## What a first-time contributor should see

Clone, install the toolchains above, run the suite. Every gate should pass on an unmodified
checkout. If any gate cannot run, it exits `1` and says which prerequisite is missing — never
exit `0`, which would report a check that silently did not happen.
