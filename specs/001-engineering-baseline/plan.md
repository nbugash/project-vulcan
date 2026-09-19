# Implementation Plan: Engineering Baseline

**Branch**: `001-engineering-baseline` | **Date**: 2026-09-18 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-engineering-baseline/spec.md`

## Summary

F000 makes the constitution's gates executable and builds the shell they measure. It delivers
four checks that currently exist only as prose — architectural boundary enforcement, resource
budget measurement on the baseline machine, prototype fidelity enforcement, and feature
sequencing — and the application shell that gives the budget and fidelity gates a real subject.

The shell is presentation and local interaction: every region the prototype composes, rendered
and responsive, with nothing behind the controls. It is built before the plugin host exists and
is re-hosted through extension points by F002. That retrofit is accepted deliberately, because
a budget measured against a placeholder measures nothing.

The approach follows from one decision: the product is Rust, and layering is expressed as
crates in a Cargo workspace rather than as folders inside one crate. A crate cannot import
what it does not declare as a dependency, so the boundary rule in Principle I becomes a
compile error rather than a lint finding. Gate 1 then verifies that the workspace dependency
graph matches the declared layering, which is a far smaller and more reliable check than
parsing imports.

The remaining three gates are measurement rather than prevention, and each needs an
environment it can trust: a constrained machine for budgets, a pinned renderer for visual
comparison, and the backlog for sequencing. Sequencing is already delivered.

## Technical Context

**Language/Version**: Rust 1.83+ (2021 edition) for the product and for gate tooling that
ships with it. Python 3.11+ for Spec Kit-adjacent scripts that already exist under
`.specify/`.

**Primary Dependencies**: GPUI (published crate, currently 0.2.2) for native GPU rendering,
used by the shell in this feature and by the client thereafter. `cargo` as build and test
driver. Capture runs under headless `sway` on the wlroots headless backend with Mesa's software
Vulkan driver and `grim`, all pinned in the comparison image. Also known and not in question for
later features: `tree-sitter` (in-process, C library with first-class Rust bindings) and
`wasmtime` (plugin host). See [User interface framework](./research.md#user-interface-framework)
and [Visual comparison environment](./research.md#visual-comparison-environment).

**Storage**: N/A for this feature. Measurement results are files under the feature's own
output directory; there is no database.

**Testing**: `cargo test` for Rust, `unittest` for the existing Python tooling. Domain and
use-case tests run without network, database or filesystem access, per Principle V.

**Target Platform**: Linux and macOS desktop. Windows is confirmed out of scope. The
measurement baseline is hardware of A18 Pro class: 6 CPU cores as 2 performance plus 4
efficiency, 8 GB RAM, integrated graphics, 120Hz display.

**Project Type**: Desktop application with a remote daemon. This feature is the repository's
tooling and verification layer, not product functionality.

**Performance Goals**: This feature does not have performance goals of its own; it measures
the product's. The complete gate suite must finish within fifteen minutes (SC-007) so that it
runs on every change rather than being avoided.

**Constraints**: Capture requires a Wayland session rather than a virtual X framebuffer,
because GPUI presents a Vulkan swapchain and X11 needs DRI3, which a machine without a GPU does
not provide. Every check must produce the same verdict locally and in the build (FR-019).
Visual comparison is exact, with no tolerance, which is only tenable inside one pinned
rendering environment (FR-023). Budget measurement must refuse to report rather than report
from an unconstrained machine (FR-005).

**Scale/Scope**: 33 functional requirements, 11 success criteria, 18 backlog subfeatures, 4
gates and the application shell. Roughly 500 lines of sequencing tooling already exist in the
repository and are adopted rather than rewritten.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Note |
| --- | --- | --- |
| I. Domain Independence | PASS | Crate-per-layer makes the rule structural. This feature builds the check itself. |
| II. Every Side Effect Is a Port | PASS | Measurement, rendering capture and process execution are ports with fakes for use-case tests. |
| III. Use Cases Own Orchestration | PASS | Each gate is a use case; the command-line entry point is an inbound adapter that maps arguments to it. |
| IV. Explicit Composition Root | PASS | One wiring module per binary. |
| V. Test at the Boundary | **RISK** | The sequencing tooling already in the repository was written before this specification, and its tests are Python `unittest` rather than the tier structure this principle defines. See Complexity Tracking. |
| VI. Runs on the Baseline Machine | **RESOLVED** | Apple Silicon runners reproduce core asymmetry and are authoritative; a Linux cgroup runner is advisory. Core count still differs and is recorded per measurement. See [Budget measurement hardware](./research.md#budget-measurement-hardware). |
| VII. Design Before Code | **VIOLATION** | Roughly 500 lines of sequencing tooling exist without a design document, written before this workflow was applied to it. See Complexity Tracking. |
| VIII. Prototype Fidelity | PASS | This feature builds both the enforcement and its subject. Every shell region uses extracted tokens only, and states the prototype omits are recorded extensions. |
| IX. Feature Map Governs Sequence | PASS | Already delivered and in force; this plan was produced through its gate. |

## Project Structure

### Documentation (this feature)

```text
specs/001-engineering-baseline/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0: decisions, rationale, alternatives
├── data-model.md        # Phase 1: entities
├── contracts/           # Phase 1: command-line and file-format contracts
├── quickstart.md        # Phase 1: how to run and verify the gates
├── architecture.md      # Phase 2: system shape
├── design.md            # Phase 2: modules, interfaces, signatures
└── checklists/
    └── requirements.md  # Specification quality checklist
```

### Source Code (repository root)

```text
crates/
├── vulcan-domain/          # Entities and rules. Depends on nothing.
├── vulcan-app/             # Use cases and ports. Depends on domain only.
├── vulcan-adapters/        # Adapter implementations. Depends on app.
├── vulcan-ui/              # Shell: an inbound adapter. Depends on app.
└── vulcan-cli/             # Composition root and inbound adapters.

tools/
├── gate-boundary/          # Gate 1: workspace dependency graph check
├── gate-budget/            # Gate 5: measurement on the constrained machine
├── gate-fidelity/          # Gate 8: token extraction, off-token lint, visual comparison
└── shell-preview/          # Runs the shell standalone for capture and measurement

.specify/extensions/featuremap/   # Gate 9: already delivered
mockups/                          # Signed-off prototype, tokens, vendored fonts and icons
reports/mockups-discrepancies/    # Recorded departures from the prototype
```

`vulcan-ui` is an adapter, not a layer of its own: the shell is an inbound adapter that maps
input to use cases, so GPUI types never reach `vulcan-app` or `vulcan-domain`. The domain and
application crates exist in this feature only far enough to prove the boundary check fails and
passes, and to give the shell somewhere to send input. They are populated by F001 and later.

## Complexity Tracking

| Violation | Why it exists | Simpler alternative rejected because |
| --- | --- | --- |
| Principle VII: sequencing tooling written before its design document | The sequence gate had to exist before the first feature could be specified in order, so it was built as governance machinery rather than product code. It is now in use and enforcing gate 9. | Deleting and rebuilding it under the workflow would discard working, tested code to satisfy a document. This plan adopts it instead, and `design.md` records it as pre-existing with its current shape documented rather than proposed. |
| Principle V: that tooling's tests do not follow the tier structure | Its 30 tests are Python `unittest` covering parsing, resolution, integrity and insertion. They are behaviour tests but were not written against domain, use-case and adapter tiers. | Restructuring them now would change no behaviour and risk breaking the gate that every later feature depends on. They are accepted as characterisation tests, which is what Principle V requires before modifying untested code. |
| The shell is built before the plugin host, contradicting "every feature is a plugin" | The budgets are the product's central claim and cannot be measured against a placeholder, so the shell must exist in the first feature, while the plugin host is F001. | Building the plugin host first would delay every budget measurement until after it, leaving the framework choice untested and the central claim unverified for two features. F002 re-hosts the shell through extension points; the retrofit is bounded because the shell is an inbound adapter and its boundary to the application layer does not move. |
| Principle VI: the measurement machine does not match the baseline's core count | Apple Silicon runners reproduce the performance and efficiency split but not the exact six-core configuration of the reference hardware. | Measuring only on symmetric Linux cores would violate FR-005 by reporting numbers from a machine the product does not target. Every measurement now records its observed core topology, and a baseline from a different topology is rejected rather than compared, so the remaining gap is visible in the report instead of hidden inside it. |


## Constitution Re-Check (post-design)

Re-evaluated after Phase 1 and Phase 2.

| Principle | Status | Note |
| --- | --- | --- |
| I. Domain Independence | PASS | `design.md` places every framework touch in `vulcan-adapters`; GPUI never appears in domain or use-case types. |
| II. Every Side Effect Is a Port | PASS | Five ports: workspace graph, constrained runner, token source, render capture, image compare. Each has a fake for use-case tests. |
| III. Use Cases Own Orchestration | PASS | The four `tools/` binaries parse arguments and map verdicts to exit codes; no branching on gate logic. |
| IV. Explicit Composition Root | PASS | `vulcan-cli` and each tool's `main` wire adapters explicitly. |
| V. Test at the Boundary | ACCEPTED RISK | Unchanged from the pre-design check. The Python tooling's thirty tests stand as characterisation tests, recorded in Complexity Tracking. |
| VI. Runs on the Baseline Machine | PASS | Resolved in research; residual core-count gap is recorded per measurement rather than hidden. |
| VII. Design Before Code | PASS for new work, ACCEPTED VIOLATION for pre-existing | This document is the design for the three new gates. The pre-existing sequencing tooling is documented as it stands in `design.md`, not presented as designed here. |
| VIII. Prototype Fidelity | PASS | The reproduction uses GPUI and extracted tokens only; the comparison is exact inside the pinned environment. |
| IX. Feature Map Governs Sequence | PASS | This plan was produced through the gate. |

No new violations were introduced by the design. The two accepted items are unchanged in
scope from the pre-design check.
