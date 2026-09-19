# Research: Engineering Baseline

**Branch**: `001-engineering-baseline` | **Date**: 2026-09-18 | **Plan**: [plan.md](./plan.md)

Every unknown in the plan's Technical Context is resolved here. Downstream artifacts
reference these headings rather than restating the reasoning.

## Layering enforcement mechanism

**Decision**: Express architectural layers as separate crates in a Cargo workspace, and
enforce Principle I by checking that the workspace dependency graph matches the declared
layering. `vulcan-domain` depends on nothing, `vulcan-app` depends on the domain,
`vulcan-adapters` depends on the app, and `vulcan-cli` is the composition root.

**Rationale**: A crate cannot import a crate it does not declare as a dependency, so the
boundary rule becomes a compile error rather than a finding reported after the fact. The gate
then verifies a handful of manifest declarations instead of parsing every import in the
repository, which is both smaller and impossible to circumvent by clever import syntax. The
compiler enforces the common case; the gate exists to catch someone adding a dependency edge
that reverses the layering.

**Alternatives considered**:

- *Module folders inside a single crate, with an import lint.* Rejected: Rust's module
  privacy does not prevent an inner module from using an outer one, so the rule would rest
  entirely on a lint that a determined edit can work around.
- *`dylint` custom lints.* Rejected as the primary mechanism: it requires maintaining a lint
  against internal compiler APIs, which is significant ongoing cost for a check the
  dependency graph already gives for free. Retained as an option later for finer rules.
- *`cargo-deny` bans.* Rejected as primary: it is built for licence and advisory policy
  against external crates, and expressing internal layering through it is a misuse that reads
  poorly.

## User interface framework

**Decision**: GPUI for the native, GPU-rendered client.

**Rationale**: It is the only candidate with a public existence proof at IDE scale under
budgets of this shape, having been built for and shipped in an editor that renders text at
frame rate. Text handling is the part of this product that is least affordable to get wrong,
and GPUI treats it as the primary case rather than as content inside a general widget system.

**Alternatives considered**:

- *iced.* Mature and well documented, but built as a general application toolkit; the text
  and editor surface would still be written by hand, which removes most of the advantage of
  adopting a framework at all.
- *egui.* Fastest route to a window and excellent for internal tooling, but immediate-mode
  rendering repaints continuously, which is in direct tension with the 1% idle processor
  budget in Principle VI.
- *Raw `wgpu`.* Total control, at the cost of writing layout, text shaping, input handling
  and accessibility from nothing. Months of work before the first prototype pixel, for
  control this feature does not need.

**Risk accepted**: GPUI's public API is young and will churn. The mitigation is Principle I:
the framework is an adapter concern, so churn is contained at the edge rather than reaching the
domain or use cases. See [Shell placement in the architecture](#shell-placement-in-the-architecture).

**Version note**: the dependency is the published crate, currently 0.2.2. Zed's main branch also
declares version 0.2.2 while containing a different renderer and an entire headless subsystem
that the published crate lacks, so the version string does not identify the code. Any future move
to the upstream branch must pin a commit, and must be recorded here as a decision rather than a
dependency bump.

## Budget measurement hardware

**Decision**: Two runners. Apple Silicon is the authoritative budget gate; a Linux runner
constrained by cgroups to 6 cores and 8 GB is an advisory second signal. Where the two
disagree, the report records both and the Apple Silicon figure decides.

**Rationale**: The baseline is hardware of A18 Pro class, whose six cores are two performance
plus four efficiency. Scheduling on asymmetric cores differs enough from symmetric cores that
a measurement taken on six equal cores is a measurement of a machine the product does not
target. Apple Silicon runners reproduce the asymmetry, which is the property that matters
most; they do not reproduce the exact core count, which matters less. The Linux runner still
earns its place because the product ships on Linux and a Linux-only regression would
otherwise go unseen until release.

**Alternatives considered**:

- *Linux cgroups alone.* Rejected: reporting symmetric measurements as baseline measurements
  is precisely what FR-005 forbids, and the gap would be invisible in the report.
- *Self-hosted ARM hardware matching the reference.* The most accurate option and the most
  expensive to own and maintain for a single-developer project. Revisit if the two runners
  diverge often enough to be untrustworthy.

**Closed at implementation, 2026-09-19, and it weakens the decision above.**

The runner is `macos-26`, the arm64 GitHub-hosted image that went generally available on
2026-02-26, pinned rather than `macos-latest` so the machine cannot move without the
repository changing. GitHub documents it as **3 arm64 vCPUs and 7 GB**, against a baseline of
six cores as two performance plus four efficiency, and 8 GB.

The core *count* gap was anticipated and is handled: every measurement records its topology and
a baseline from a different topology is rejected rather than compared.

The *asymmetry* gap was not. The rationale above rests on hosted Apple Silicon reproducing the
performance and efficiency split, and a three-vCPU virtual machine most likely does not: guest
vCPUs are typically uniform, so `hw.perflevel1.logicalcpu` is expected to be absent and the
observed topology to read `3P+0E`. If that is what the runner reports, then the property this
decision was made for is not present, and calling the Apple Silicon figure *authoritative*
claims more than the machine supports.

This is not guessed at in the report: the workflow's `Observed machine` step prints the
architecture, both performance levels and the memory size before the gate runs, so the first
run on this image settles it in the log. Until it does, treat an Apple Silicon figure as the
better of two approximations rather than as the baseline's number.

**If the runner reports `3P+0E`**, the options, in ascending cost:

1. *Keep both runners, drop the word authoritative.* Two approximations, each recording its own
   topology, neither claiming to be the baseline. Free, honest, and weaker than the
   constitution currently promises.
2. *`macos-26-xlarge`.* Also arm64 and a larger machine, so more likely to expose real
   performance and efficiency cores. Billed per minute, and still not six cores as 2+4.
3. *A self-hosted runner on hardware of A18 Pro class.* The only option that measures what
   Principle VI actually describes. Previously rejected as disproportionate for a
   single-developer project; it is the honest answer if the budgets are to be enforced as
   written.

## Gate tooling language

**Decision**: Rust for gates 1, 5 and 8, built as workspace members under `tools/`. Python
stays for gate 9, the existing `featuremap` extension.

**Rationale**: The budget gate has to instrument the product, and measuring the longest
interface-thread task from inside the process yields an exact number where an external
observer can only sample and infer. Putting that gate in the product's language makes it a
library call rather than a protocol. The boundary and fidelity gates follow it for one
toolchain and one test runner. Gate 9 is Spec Kit-adjacent rather than product-adjacent, it
already works, and it has thirty passing tests; rewriting it would discard working code to
buy consistency that measures nothing.

**Alternatives considered**:

- *Everything in Python.* Rejected because in-process instrumentation would be impossible and
  the budget gate would fall back to sampling.
- *Everything in Rust, including a rewrite of `featuremap`.* Rejected under the constitution's
  own rule that modifying working, tested code requires characterisation tests first; the
  rewrite would cost that effort and change no behaviour.

## Shell placement in the architecture

**Decision**: The shell is an inbound adapter, living in `crates/vulcan-ui` and declared in the
Adapters layer. It maps input to use cases and renders their output. GPUI types never appear in
`vulcan-app` or `vulcan-domain`.

**Rationale**: Principle I makes the boundary structural, and gate 1 enforces it from the first
commit. Placing the shell in Adapters means the framework choice is contained at the edge: if
GPUI's young API churns, or is replaced entirely, the domain and application layers do not move.
It also bounds the F002 retrofit, because re-hosting the shell through plugin extension points
changes how the adapter is instantiated, not where the boundary sits.

**Alternatives considered**:

- *A `vulcan-ui` layer of its own, above Adapters.* Rejected: it would add a layer to the rule
  set for one crate, and the shell has no property that the Adapters layer does not already
  describe.
- *The shell inside `vulcan-cli`.* Rejected: the composition root wires things together and
  should not also be the largest adapter in the system.

## Behaviour boundary for the shell

**Decision**: Controls respond visually and route to a no-op command sink. No control performs
work behind it in this feature.

**Rationale**: The shell exists so the budget and fidelity gates have a real subject, not so the
product functions. A control that half-acts is worse than one that does nothing, because the
feature that later owns that behaviour cannot distinguish a stub from a defect. A single sink
also gives F002 one place to replace when commands become real.

**Alternatives considered**:

- *Wire controls to the behaviour that exists.* Rejected: almost nothing exists yet, so the
  result would be a few real controls among many stubs, with no way to tell which is which.
- *Disable controls entirely.* Rejected: a disabled control renders differently from an enabled
  one, so the fidelity comparison would diverge from the prototype it is meant to match.

## Prototype extension enforcement

**Decision**: States the prototype does not depict are recorded as prototype extensions with a
sign-off status, and the build fails when the shell reaches a state that has no record.

**Rationale**: The prototype composes one viewport, one theme, and only the states it draws.
Resizing the window, any empty or error state, and every hover or focus treatment it omits are
design decisions someone has to make. Recording them before implementation is what keeps that
decision visible to the designer rather than settled silently in code.

**Alternatives considered**:

- *Record extensions after implementation.* Rejected: the record then documents what was built
  rather than what was agreed, which inverts its purpose.
- *Treat omitted states as free choices.* Rejected: it is precisely the accumulation of small
  free choices that Principle VIII exists to prevent.

## Visual comparison environment

**Decision**: Authoritative visual comparison runs inside one pinned container that starts a
headless `sway` session on the wlroots headless backend, renders through Mesa's software
Vulkan driver, and captures with `grim` through `wlr-screencopy`. The same image runs
locally, so the local verdict and the build verdict agree.

**Rationale**: GPUI presents a Vulkan swapchain, and how that swapchain reaches a readable
buffer decides whether capture is possible at all. On X11 the handoff requires DRI3, which
this class of machine does not provide: a cloud instance with no GPU has no DRM render node,
and neither Xvfb nor Xorg's dummy driver implements DRI3. Measured on the target instance,
every X11 route produced a correctly sized window, no error output, and an entirely black
image. Wayland presents through `wl_shm`, which is shared memory and needs no DRM device, so
software rendering works with CPU alone. wlroots' headless backend also advertises a virtual
`wl_seat`, which GPUI requires and which weston's headless backend does not provide.

Pinning remains necessary for a different reason than the mechanism: exact comparison with no
tolerance means the compositor version, the software rasteriser and the typefaces all decide
pixels, so all three are fixed in the image.

**Alternatives considered**, each ruled out by measurement on the target instance rather than
by reasoning:

- *Xvfb.* Window created at the correct size, no errors, 250,000 black pixels. No DRI3.
- *Xvfb with a compositor (`xcompmgr`).* Identical result; compositing does not create the
  buffer sharing that was missing.
- *Xorg with `xserver-xorg-video-dummy`.* Starts cleanly and reports Present, Composite and
  GLX, but not DRI3. Identical black capture. The dummy driver emulates a framebuffer, which
  is what an X11 drawing application needs; GPUI is not one.
- *X11 forwarding to a developer machine.* Same mechanism, relocated. Solves interactive
  inspection, not automated capture.
- *weston headless.* GPUI panics on `seat.unwrap()`: the backend advertises no `wl_seat`.
- *A GPU instance type.* Would supply a render node and DRI3, and would work. Rejected because
  it makes every fidelity run cost GPU hours, and because a driver version then becomes part
  of the reference image's identity: upgrading the machine image would diff every screen.
- *Rendering offscreen through a headless renderer.* The cleanest mechanism, and unavailable:
  the published `gpui` crate has no such path, and on Zed's main branch the only
  implementation of `PlatformHeadlessRenderer` is `MetalHeadlessRenderer`, for macOS. Revisit
  if a Linux implementation lands upstream, since it would remove the compositor entirely.

**Consequence**: capture requires no GPU, no display, no display forwarding and no change of
instance type, and it works against the published `gpui` crate rather than a pinned commit of
an upstream branch.

**Not valid for budgets**: this environment composites in software and renders in software.
It measures appearance, never performance. Gate 5 continues to run on the hardware described
in Budget measurement hardware.

## Prototype token sources

**Decision**: Extraction reads two sources: the custom properties in
`mockups/_ds/nocturne-*/styles.css`, and the values documented in `mockups/assets.md` that are
not custom properties. Output is a generated Rust module of constants plus a machine-readable
list used by the off-token lint.

**Rationale**: The design system is deliberately monochrome, so the five semantic state
colours and the nine syntax colours were added outside it and exist only in the manifest.
Reading the stylesheet alone would miss fourteen values that the product is nonetheless
required to use rather than invent, and the off-token lint would then reject them as
violations the moment anyone used them correctly.

**Alternatives considered**:

- *Stylesheet only.* Rejected for the reason above.
- *Hand-transcribing tokens into Rust.* Rejected: a transcription is a fork of the design
  system that diverges silently on the first prototype change. Extraction must be re-runnable
  so a prototype change surfaces as a diff.

## Network profile simulation

**Decision**: Round-trip delay is injected with `tc netem` on a loopback interface inside the
measurement environment, at 0, 10, 30 and 80 milliseconds, with 80 as the reference for the
completion budget.

**Rationale**: The budgets that involve a remote host are stated relative to round trip, so
the measurement must be able to produce a round trip on demand rather than waiting for a real
network. Loopback injection keeps the measurement deterministic and removes the internet as a
source of variance.

**Alternatives considered**:

- *Measuring against a real remote host.* Rejected for the measurement gate: real links vary
  between runs, which makes regressions indistinguishable from weather. Retained for
  end-to-end validation in later features.
- *Simulating delay in application code.* Rejected: it would measure the simulation rather
  than the protocol, and would not capture effects such as connection setup or congestion.

## Pre-existing sequencing tooling

**Decision**: Adopt the `featuremap` extension as it stands. Its thirty tests are accepted as
characterisation tests. `design.md` documents its current shape rather than proposing one.

**Rationale**: It was built before this workflow applied to it, it is installed, and gate 9
depends on it. The constitution requires characterisation tests before modifying untested
code; this code is tested, so the requirement is already met in substance. Recording it as
pre-existing is honest, where silently presenting it as designed here would not be.

**Alternatives considered**:

- *Rebuild it under the workflow.* Rejected: it would discard working, tested code to satisfy
  a document, and would put every later feature's sequencing at risk during the rewrite.
- *Leave it undocumented.* Rejected: Principle VII exists so that the shape of the system is
  recoverable, and an undocumented component that every feature passes through is the worst
  possible place for that gap.
