# Gates

Four checks stand between a change and the claims this product makes. Each is a workspace
binary except gate 9, which is a vendored Python extension.

Every gate uses the same three exit codes, and the third is the one that matters:

| Exit | Meaning |
|------|---------|
| `0` | Judged, and passed |
| `2` | Judged, and failed |
| `1` | **Could not judge.** A prerequisite was missing, so nothing was checked |

A gate that cannot run exits `1`, never `0`. Reporting a pass for a check that did not happen
is the one failure mode that makes the whole suite worthless, because the green result is
believed.

## Gate 1 — architectural boundaries

```bash
cargo run -p gate-boundary
```

Reads the workspace dependency graph and judges it against the declared layering in each
crate's `[package.metadata.vulcan]`. A crate without a declared layer fails rather than
defaulting, so adding a crate is a deliberate act.

Refuses (`1`) when it finds source under `crates/` or `tools/` in a language it has no
boundary rule for. It governs Rust. Manifests, documentation and shell scripts are ignored
because they carry no imports and so no direction to enforce; that ignore list is the gate's
blind spot and stays short.

**Prove it fails.** Add `vulcan-adapters` to `crates/vulcan-domain/Cargo.toml` and re-run:
exit `2`, naming both crates and the manifest line.

## Gate 5 — resource budgets

```bash
cargo run -p gate-budget -- --runner apple-silicon    # authoritative
cargo run -p gate-budget -- --runner linux-cgroup     # advisory
```

Measures the running shell against Principle VI's budgets. The shell measures itself: an
external sampler at 100 Hz cannot see an 8 ms budget, so the frame loop records its own
timings and the runner only collects them.

Refuses (`1`) when it cannot enforce the constraints — on a machine where
`/sys/fs/cgroup/cgroup.subtree_control` is not writable, for instance — and writes no report.
Refusing is not failing, and neither is passing.

Only the Apple Silicon result decides, because it reproduces the baseline's performance and
efficiency core split. Every measurement records the core topology it was taken on, and a
baseline from a different topology is rejected rather than compared.

### The budget exception path

There isn't one that works by changing the number.

If a measurement exceeds its budget, the options are to make it faster, to move the work off
the path being measured, or to take the overage to the constitution as an amendment with its
own rationale and sign-off. **Raising a budget to match a measured value is not an exception,
it is deleting the check.** The budget describes what the product promises on the baseline
machine; a number edited to match today's measurement promises nothing.

Degradation to hold a budget is bounded by Principle VIII's precedence ladder: shadows,
transitions and animation may go; layout, type scale and colour may not.

## Gate 8 — prototype fidelity

```bash
cargo run -p gate-fidelity -- extract          # design values from the prototype
cargo run -p gate-fidelity -- lint             # no invented values in our source
cargo run -p gate-fidelity -- discrepancies    # every discrepancy is triaged
./tools/gate-fidelity/run-in-pinned-env.sh compare
```

**`extract`** reads the stylesheet's custom properties and the values documented only in
`mockups/assets.md`, and writes both `out/tokens.json` and
`crates/vulcan-ui/src/generated_tokens.rs`. Deterministic: run it twice and diff.

**`lint`** reads the reproduction's own source and reports any design value that appears
nowhere in the prototype. This is the only gate that reads the prototype as the authority.
The comparison cannot do it: the reference is captured *from* the shell, so a value invented
before the capture is baked into the reference and compares clean forever.

A literal passes if the design system names it, or if the prototype uses it unnamed — a
padding written inline in the prototype is legitimate without ever being a token. It fails
only when the reproduction invented it.

**`discrepancies`** fails on any document under `reports/mockups-discrepancies/` still marked
`Untriaged`, and on a `BacklogEntry` naming a feature absent from `specs/features-map.md`. It
also fails when a design document restates a discrepancy's body instead of linking to it: a
restatement is a copy that stops tracking the original.

**`compare`** renders the shell and compares it to the stored reference with **no tolerance**.
Any differing pixel fails, with a visual report rather than only a count. It refuses (`1`)
outside the pinned environment, because an exact comparison taken elsewhere is not weaker
evidence, it is different evidence.

When the prototype itself changes, `compare` reports a **stale reference** rather than a
regression, naming the digest that moved. Re-approve the interface; do not re-baseline to make
it green.

### The capture environment renders in software

The pinned environment runs a headless `sway` session on the wlroots headless backend, with
Mesa's software Vulkan driver (lavapipe) and `grim`. Nothing is rasterised by a GPU and
nothing is composited by one.

**It therefore measures appearance only, and is never a source of budget figures.** A frame
time taken here is a property of lavapipe on the build machine, not of the product on the
baseline machine. Gate 5 exists for that, and it runs elsewhere.

Why a compositor rather than a virtual X framebuffer: GPUI presents a Vulkan swapchain, and on
X11 that handoff needs DRI3, which a machine with no GPU does not provide. Capture under Xvfb
or Xorg's dummy driver returns a uniformly black image with nothing on stderr to explain it.
Wayland presents through shared memory instead. If a capture ever comes back black, check that
the session is Wayland before looking anywhere else.

## Gate 9 — feature sequencing

```bash
python3 .specify/extensions/featuremap/scripts/python/feature_map.py verify
python3 .specify/extensions/featuremap/scripts/python/feature_map.py resolve
python3 -m pytest .specify/extensions/featuremap/tests/ -q
```

Pre-existing, vendored from the spec-kit fork that maintains it, and enforced through the
`before_specify` hook so a feature cannot be specified out of order. Its 30 behaviour tests
are vendored beside it, because the copy in this repository is the one that decides.

**Prove it blocks.** `resolve F001` exits `2` while F000 is unfinished, naming F000.

## Running the suite

```bash
cargo test --workspace
cargo run -p shell-preview -- --smoke
cargo run -p gate-boundary
cargo run -p gate-fidelity -- extract
cargo run -p gate-fidelity -- lint
cargo run -p gate-fidelity -- discrepancies
cargo run -p gate-budget -- --runner apple-silicon
./tools/gate-fidelity/run-in-pinned-env.sh compare
python3 .specify/extensions/featuremap/scripts/python/feature_map.py verify
python3 -m pytest .specify/extensions/featuremap/tests/ -q
```

### Measured runtime

SC-007 requires the suite under fifteen minutes, because a suite people avoid protects
nothing. Measured on the development machine with a warm `target/`:

| Step | Time |
|------|------|
| `cargo test --workspace` (87 tests) | 2–3 s |
| Gate 1, boundaries | < 1 s |
| Gate 8, extract / lint / discrepancies | < 1 s each |
| Gate 9, 30 tests plus verify and resolve | < 1 s |
| Gate 5, budgets | ~1 s to refuse; longer where it can measure |
| Gate 8, compare, including the pinned session | ~6 s per capture |

**Total: well under one minute warm**, against a fifteen-minute budget.

Two honest caveats. A cold build dominates everything above — GPUI and its dependencies take
minutes to compile from scratch, and that cost is the build cache's, not the suite's. And the
compare figure is one pinned-session round trip: while the reference is stale the gate
short-circuits before capturing, so it currently returns faster than it will once the
reference is current.

### Measured flake rate

SC-006 requires the comparison to be stable: a gate that fails at random is one people learn
to re-run until it passes.

Twenty consecutive comparisons against an unchanged reference, in the pinned session:

```
20 runs in 188s: pass=20 fail=0 other=0
```

**Zero failures unrelated to a change.** The environment renders and composites in software
with no GPU, no display server timing and no font fallback, which is why the output is
reproducible enough to compare with no tolerance at all. Two independent captures of the same
shell are byte-identical.

### The capture reaches no network

A reference that silently depended on a CDN would be reproducible only while that CDN was up,
and would change without anything in the repository changing.

Rather than disable networking — which needs privileges the build machine does not grant — the
capture is traced and its syscalls inspected:

```
socket() calls by family:   5 × AF_UNIX
connect() to AF_INET:       none
```

Every socket is the Wayland compositor's own IPC. The capture attempts no network connection
at all, and the reference it produces is byte-identical to one captured on a networked
machine. Typefaces and icons are vendored under `mockups/fonts/` and `mockups/icons/` and load
from the repository.

## Before anything else

```bash
make setup           # report what this machine is missing
make setup-install   # and set up the Python tooling it can
```

It reports rather than installs: system packages are printed as commands to run,
because a build script that runs `sudo` against a package manager is one that can
break a machine it does not understand.

## Verifying on a Mac

Some things cannot be answered from Linux, and the most important of them is
whether the budget gate's central claim is true. Gate 5 calls the Apple Silicon
runner *authoritative* because that hardware reproduces the baseline's
asymmetric cores — two performance plus four efficiency — and that has never
been checked on a machine that has any.

```bash
make macos-verify          # the checks that need no privilege
make macos-verify-sudo     # also the latency profiles
```

Nine checks, each writing `reports/macos/verification-<check>-<STATUS>-<run>.json`.
Every file from one run shares a stamp, and the directory holds one run at a
time: it is cleared at the start, after the prerequisites are checked, so a run
that bails on a missing prerequisite leaves the previous results alone.

### What it needs

The script checks all of this before doing any work, because a missing
prerequisite should cost a second rather than eight minutes of compiling.

| Requirement | Why | If it is missing |
|---|---|---|
| A Mac with Apple Silicon | An Intel Mac cannot answer the topology question | The script stops |
| Xcode Command Line Tools | clang for the `cc` crate, libclang for `bindgen` | `xcode-select --install` |
| **Full Xcode** | GPUI compiles its Metal shaders during the build, and the `metal` compiler is not in the Command Line Tools | See below |
| rustup | Installs the toolchain `rust-toolchain.toml` pins | [rustup.rs](https://rustup.rs) |
| About 10 GB free | Most of it the build | Free some space |
| A logged-in desktop session | GPUI needs a window server to open a window | Only the `shell-render` check fails; the rest still run |
| `sudo` | `dnctl` and `pfctl` need it | Only with `--sudo` |

Running over SSH is fine for eight of the nine checks. The one that opens a
window will fail, and says so in its own output rather than leaving it to be
guessed at.

### The Metal compiler

The Command Line Tools are not enough, whatever else they provide. GPUI compiles
its shaders in a build script, and `metal` ships only with Xcode. On Xcode 26 it
is a separate download even then.

```bash
# 1. Install Xcode from the App Store
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
xcodebuild -runFirstLaunch
xcodebuild -downloadComponent metalToolchain    # Xcode 26 and later

xcrun -f metal      # should print a path
```

`make setup` checks exactly this, so the answer costs a second rather than a
failed build.

### Sending the results back

```bash
git add reports/macos && git commit -m 'macOS verification' && git push
```

`reports/macos/` is deliberately not ignored, unlike the other directories under
`reports/`: these files are evidence about a machine nobody else can reach, so
they are the one measurement output worth keeping in the history.
