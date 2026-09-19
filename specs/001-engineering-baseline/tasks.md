# Tasks: Engineering Baseline

**Input**: Design documents from `/specs/001-engineering-baseline/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, architecture.md, design.md

**Tests**: Included. Principle V makes test-first non-negotiable for domain and use-case code,
and the specification's acceptance scenarios require each gate to be seen failing, not only
passing.

**Organization**: Tasks are grouped by user story so each gate can be built, run and trusted
on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelisable — different files, no dependency on unfinished work
- **[US1..US5]**: the user story in `spec.md` the task serves

## Path Conventions

Paths come from `design.md` Module & File Layout. Rust crates live under `crates/`, gate
binaries under `tools/`, and the pre-existing Python sequencing tooling under
`.specify/extensions/featuremap/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: The workspace whose shape is itself the boundary rule.

- [X] T001 Pin the toolchain in `rust-toolchain.toml` (Rust 1.83+, 2021 edition) per plan.md Technical Context
- [X] T002 Create the workspace root `Cargo.toml` declaring members `crates/*` and `tools/*`
- [X] T003 [P] Create `crates/vulcan-domain/` with `src/lib.rs` and no dependencies
- [X] T004 [P] Create `crates/vulcan-app/` with `src/ports/` and `src/use_cases/`, depending only on `vulcan-domain`
- [X] T005 [P] Create `crates/vulcan-adapters/` with `src/manifest/`, `src/measurement/`, `src/rendering/`, `src/tokens/`, depending on `vulcan-app`
- [X] T006 [P] Create `crates/vulcan-cli/` as the composition root
- [X] T006a [P] Create `crates/vulcan-ui/` for the shell, declared in the Adapters layer, depending on `vulcan-app`
- [X] T007 [P] Create the four binaries `tools/gate-boundary/`, `tools/gate-budget/`, `tools/gate-fidelity/`, `tools/shell-preview/` as workspace members
- [X] T008 Declare each crate's layer in its `Cargo.toml` metadata, using the `Domain`/`Application`/`Adapters`/`Composition`/`Tooling` values from data-model.md
- [X] T009 [P] Pinned comparison environment in `tools/gate-fidelity/pinned-env/Containerfile`: headless `sway`, `grim`, Mesa software Vulkan and the vendored typefaces, all version-pinned
- [X] T010 [P] `tools/gate-fidelity/run-in-pinned-env.sh` starts a headless wlroots session inside that image and runs the gate, borderless so the capture holds only the product's pixels

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The verdict vocabulary and port surface every gate depends on.

**⚠️ CRITICAL**: No gate work begins until this phase is complete.

- [X] T011 Write failing tests for verdict-to-exit-code mapping in `crates/vulcan-domain/src/verdict.rs` tests: judged-and-passed is `0`, judged-and-failed is `2`, could-not-judge is `1`
- [X] T012 Implement `GateVerdict` and `GateError` in `crates/vulcan-domain/src/verdict.rs`, making `CouldNotJudge` structurally distinct from a failing verdict
- [X] T013 [P] Define `WorkspaceGraphPort` in `crates/vulcan-app/src/ports/workspace_graph.rs` with the signature from design.md
- [X] T014 [P] Define `ConstrainedRunnerPort` in `crates/vulcan-app/src/ports/constrained_runner.rs`
- [X] T015 [P] Define `TokenSourcePort` in `crates/vulcan-app/src/ports/token_source.rs`
- [X] T016 [P] Define `RenderCapturePort` in `crates/vulcan-app/src/ports/render_capture.rs`
- [X] T017 [P] Define `ImageComparePort` in `crates/vulcan-app/src/ports/image_compare.rs`
- [X] T018 Build the shared port contract-test harness in `crates/vulcan-app/tests/contract/mod.rs`, so one suite runs against every adapter and every fake, per Principle V
- [X] T019 [P] Implement in-memory fakes for all five ports in `crates/vulcan-app/tests/fakes/`, each passing the contract suite
- [X] T020 Implement shared report emission (`--json`, `--report <path>`) in `crates/vulcan-adapters/src/reporting.rs` per contracts/gates-cli.md
- [X] T021 Implement the inbound-adapter argument and exit-code shell in `crates/vulcan-cli/src/gate_entry.rs`, used by all four binaries

**Checkpoint**: Ports, fakes, verdicts and exit codes exist. Gates can now be built in parallel.

---

## Phase 3: User Story 1 - Architectural violations caught before merge (Priority: P1) 🎯 MVP

**Goal**: A dependency edge pointing the wrong way fails the build, naming the file and the boundary.

**Independent Test**: Add a dependency on `vulcan-adapters` to `crates/vulcan-domain/Cargo.toml`, run the gate, see exit `2` naming both crates and the manifest line; remove it and see exit `0`.

### Tests for User Story 1

- [X] T022 [P] [US1] Failing domain tests for `LayerRule` in `crates/vulcan-domain/tests/layer_rule.rs`: inward edges permitted, outward rejected, same-layer permitted, undeclared layer rejected
- [X] T023 [P] [US1] Failing use-case tests for `CheckBoundaries` in `crates/vulcan-app/tests/check_boundaries.rs`, using the fake graph port, asserting every offending edge is named
- [X] T024 [P] [US1] Failing contract test for `WorkspaceGraphPort` in `crates/vulcan-app/tests/contract/workspace_graph.rs`, run against both fake and real adapter

### Implementation for User Story 1

- [X] T025 [P] [US1] Implement the `LayerEdge` entity in `crates/vulcan-domain/src/layer_edge.rs` per data-model.md
- [X] T026 [US1] Implement `LayerRule` in `crates/vulcan-domain/src/layer_rule.rs`, deciding `permitted` from the two declared layers
- [X] T027 [US1] Implement the `CheckBoundaries` use case in `crates/vulcan-app/src/use_cases/check_boundaries.rs`
- [X] T028 [US1] Implement `CargoManifestAdapter` in `crates/vulcan-adapters/src/manifest/cargo_manifest.rs`, reading declared layers and dependency edges, failing on a crate with no declared layer
- [X] T029 [US1] Detect dependency cycles in `crates/vulcan-adapters/src/manifest/cargo_manifest.rs` and report them as a distinct failure from a reversed edge
- [X] T030 [US1] Implement `tools/gate-boundary/src/main.rs`: parse `--manifest-path`, `--json`, `--report`, map verdict to exit code
- [X] T031 [US1] Integration test in `tools/gate-boundary/tests/violation_detected.rs` reproducing the quickstart scenario end to end
- [X] T032 [US1] Wire `gate-boundary` into the build pipeline on the Linux runner

**Checkpoint**: Gate 1 enforces Principle I on every change. This is the MVP: the rule every later feature is written under is now mechanical.

---

## Phase 4: User Story 2 - Budgets measured on representative hardware (Priority: P1)

**Goal**: Every constitutional budget is measured on baseline-class hardware, and a breach fails the build.

**Independent Test**: Run the suite against a placeholder binary and see all thirteen metrics reported with verdicts; run it unconstrained and see exit `1` with nothing written; inflate a metric artificially and see exit `2` naming metric, budget and measured value.

### Tests for User Story 2

- [X] T033 [P] [US2] Failing domain tests for `BudgetVerdict` in `crates/vulcan-domain/tests/budget_verdict.rs`: pass, fail, and the ten-percent regression boundary
- [X] T034 [P] [US2] Failing use-case tests for `MeasureBudgets` in `crates/vulcan-app/tests/measure_budgets.rs`, including the refuse-when-unenforceable path producing `CouldNotJudge`
- [X] T035 [P] [US2] Failing contract test for `ConstrainedRunnerPort` in `crates/vulcan-app/tests/contract/constrained_runner.rs`, run against fake, Linux and Apple Silicon adapters
- [X] T036 [P] [US2] Failing test asserting a run omitting any metric fails rather than reporting a partial set, in `crates/vulcan-app/tests/measure_budgets.rs`

### Implementation for User Story 2

- [X] T037 [P] [US2] Implement the `BudgetMeasurement` entity in `crates/vulcan-domain/src/budget_measurement.rs` per data-model.md
- [X] T038 [US2] Implement `BudgetVerdict` in `crates/vulcan-domain/src/budget_verdict.rs`, with budgets compiled in so a failing build cannot be fixed by editing a threshold
- [X] T039 [US2] Implement the `MeasureBudgets` use case in `crates/vulcan-app/src/use_cases/measure_budgets.rs`
- [X] T040 [US2] Implement in-process instrumentation in `crates/vulcan-adapters/src/measurement/instrument.rs`, reporting frame and task timings exactly rather than by sampling
- [X] T041 [P] [US2] Implement `LinuxCgroupRunner` in `crates/vulcan-adapters/src/measurement/linux_cgroup.rs`, asserting limits are applied rather than requested
- [X] T042 [P] [US2] Implement `AppleSiliconRunner` in `crates/vulcan-adapters/src/measurement/apple_silicon.rs`, reading and recording observed performance and efficiency core counts
- [X] T043 [US2] Implement round-trip injection at 0, 10, 30 and 80ms in `crates/vulcan-adapters/src/measurement/netem.rs` per research.md
- [X] T044 [US2] Implement the measurement report store in `crates/vulcan-adapters/src/measurement/report_store.rs` per contracts/file-formats.md, recording runner, authority and core topology
- [X] T045 [US2] Implement baseline loading and rejection of baselines from a different core topology in `crates/vulcan-adapters/src/measurement/baseline.rs`
- [X] T046 [US2] Implement `tools/gate-budget/src/main.rs`: `--runner`, `--rtt`, `--baseline`, exit `1` when constraints are unenforceable
- [X] T047 [US2] Integration test in `tools/gate-budget/tests/refuses_unconstrained.rs` asserting no measurements are written when constraints fail
- [X] T048 [US2] Wire `gate-budget` into the build on both runners, Apple Silicon authoritative and Linux advisory

**Checkpoint**: Every budget in Principle VI is measured and enforced on hardware that resembles the target.

---

## Phase 5: User Story 3 - The signed-off interface is enforced (Priority: P2)

**Goal**: Tokens are extracted from the prototype, off-token values are rejected, and any pixel difference from the signed-off composition fails the build.

**Independent Test**: Run extraction twice and confirm byte-identical output covering both sources; add a literal colour and see exit `2` naming value and line; shift one spacing value by a pixel and see the comparison fail with a visual report.

### Tests for User Story 3

- [X] T049 [P] [US3] Failing domain tests for `DesignValue` in `crates/vulcan-domain/tests/design_value.rs`: name validation, and duplicate names across sources rejected rather than merged
- [X] T050 [P] [US3] Failing use-case tests for `ExtractTokens`, `LintOffToken` and `CompareFidelity` in `crates/vulcan-app/tests/fidelity.rs`
- [X] T051 [P] [US3] Failing contract tests for `TokenSourcePort`, `RenderCapturePort` and `ImageComparePort` in `crates/vulcan-app/tests/contract/fidelity.rs`
- [X] T052 [P] [US3] Failing test asserting extraction is deterministic across two runs over an unchanged prototype

### Implementation for User Story 3

- [X] T053 [P] [US3] Implement the `DesignValue` entity in `crates/vulcan-domain/src/design_value.rs` per data-model.md
- [X] T054 [US3] Implement `StylesheetAndManifestAdapter` in `crates/vulcan-adapters/src/tokens/stylesheet.rs`, reading custom properties from `mockups/_ds/nocturne-*/styles.css` and the fourteen values documented only in `mockups/assets.md`
- [X] T055 [US3] Implement the `ExtractTokens` use case in `crates/vulcan-app/src/use_cases/extract_tokens.rs` with output sorted by name
- [X] T056 [US3] Emit the generated token module for product use in `crates/vulcan-adapters/src/tokens/generated.rs`, regenerated rather than hand-edited
- [X] T057 [US3] Implement the `LintOffToken` use case in `crates/vulcan-app/src/use_cases/lint_off_token.rs` reporting value, file and line
- [X] T058 [US3] Implement the source scanner in `crates/vulcan-adapters/src/tokens/scanner.rs`
- [X] T059 [US3] Implement `tools/shell-preview/src/main.rs`, running the shell standalone so the fidelity and budget gates have a subject to capture and measure
- [X] T060 [US3] Implement `GpuiCaptureAdapter` in `crates/vulcan-adapters/src/rendering/gpui_capture.rs`: render the shell under the headless session and capture through `wlr-screencopy`, refusing outside the pinned environment
- [X] T060a [US3] Capture the surface rather than the screen in `crates/vulcan-adapters/src/rendering/gpui_capture.rs`, so no compositor decoration or background enters the reference
- [X] T061 [P] [US3] Implement `ExactImageComparator` in `crates/vulcan-adapters/src/rendering/exact_compare.rs` with no tolerance
- [X] T062 [US3] Implement the fidelity reference store in `crates/vulcan-adapters/src/rendering/reference_store.rs` per contracts/file-formats.md, recording typefaces and both digests
- [X] T063 [US3] Refuse `capture-reference` while any declared typeface is absent from the repository, in `crates/vulcan-app/src/use_cases/capture_reference.rs`
- [X] T064 [US3] Distinguish a stale reference from a regression in `crates/vulcan-app/src/use_cases/compare_fidelity.rs`, using the prototype and environment digests
- [X] T065 [US3] Produce a visual report on comparison failure in `crates/vulcan-adapters/src/rendering/visual_report.rs`
- [X] T066 [US3] Implement `tools/gate-fidelity/src/main.rs` with `extract`, `lint`, `compare` and `capture-reference`
- [X] T067 [US3] Capture the first fidelity reference inside the pinned environment, now that Inter and Phosphor are vendored and headless capture is verified
- [X] T068 [US3] Wire `gate-fidelity` into the build, running `compare` inside the pinned image
- [X] T110 [US3] Capture the fidelity reference a second time with networking disabled in the
      pinned environment (`--network none`) and assert the two captures are byte-identical,
      proving the interface renders from vendored assets alone per FR-032

**Checkpoint**: The prototype is enforced mechanically rather than remembered.

---

## Phase 6: User Story 4 - Features cannot be started out of order (Priority: P3)

**Goal**: Confirm and document the sequencing gate that is already delivered.

**Independent Test**: Request a feature with an unfinished dependency and confirm the refusal names the blocker and creates nothing.

- [X] T069 [US4] Wire `python3 .specify/extensions/featuremap/scripts/python/test_feature_map.py` into the build so its thirty tests run on every change
- [X] T070 [US4] Add an integration check in the build asserting `feature_map.py resolve F001` exits `2` while F000 is unfinished
- [X] T071 [US4] Record the extension's current shape in `design.md` as pre-existing rather than proposed, closing the Principle VII item in plan.md Complexity Tracking

**Checkpoint**: All four gates are enforced and none depends on anybody remembering to run it.

---

## Phase 6b: User Story 5 - The interface exists and responds (Priority: P1)

**Goal**: Every region the prototype composes is rendered and responds to the user, with nothing
behind the controls.

**Independent Test**: Start the application, compare against the prototype and see no pixel
difference; then resize a panel, switch a tab, open and filter the palette, and change density,
each responding within the frame budget.

**Scope boundary**: presentation and local interaction only. Opening a file, contacting a
language server, executing a command and real terminal output belong to the features that own
them. A control that appears to act and does nothing is correct here; one that half-acts is not.

### Tests for User Story 5

- [X] T078 [P] [US5] Failing tests for layout arithmetic in `crates/vulcan-ui/tests/layout.rs`: fixed chrome heights and rail width from the prototype manifest
- [X] T079 [P] [US5] Failing tests for density resolution in `crates/vulcan-ui/tests/density.rs`, covering compact, default and roomy for every affected dimension
- [X] T080 [P] [US5] Failing tests for palette filtering in `crates/vulcan-ui/tests/palette.rs`, over a fixed in-memory list
- [X] T081 [P] [US5] Failing test asserting no design literal appears in `crates/vulcan-ui/`, complementing the gate 8 lint

### Implementation for User Story 5

- [X] T082 [US5] Implement the window and root layout in `crates/vulcan-ui/src/shell.rs`, at the prototype's viewport
- [X] T083 [P] [US5] Implement the toolbar and remote banner in `crates/vulcan-ui/src/chrome/toolbar.rs` at 46px and 26px
- [X] T084 [P] [US5] Implement the rail in `crates/vulcan-ui/src/chrome/rail.rs` at 44px, with its icon set from the vendored Phosphor subset
- [X] T085 [P] [US5] Implement the status bar in `crates/vulcan-ui/src/chrome/status_bar.rs` at 26px, including the performance readout variants
- [X] T086 [US5] Implement tool windows in `crates/vulcan-ui/src/panels/tool_window.rs`, resizable and collapsible within the density widths
- [X] T087 [US5] Implement the dock in `crates/vulcan-ui/src/panels/dock.rs` with its header, tab strip and the height cap the prototype declares
- [X] T088 [P] [US5] Implement the tab strip and breadcrumbs in `crates/vulcan-ui/src/editor/tabs.rs` at 34px and 24px
- [X] T089 [US5] Implement the editor surface presentation in `crates/vulcan-ui/src/editor/surface.rs`: gutter, line numbers, syntax colouring from the token palette, inlay hints
- [X] T090 [P] [US5] Implement the completion popup in `crates/vulcan-ui/src/editor/completion.rs`, in both list and detail variants
- [X] T091 [US5] Implement the command palette in `crates/vulcan-ui/src/overlays/palette.rs`: open, filter, dismiss
- [X] T092 [P] [US5] Implement the run configuration dialog in `crates/vulcan-ui/src/overlays/run_config.rs`
- [X] T093 [P] [US5] Implement the language pack dialog in `crates/vulcan-ui/src/overlays/packs.rs`
- [X] T094 [P] [US5] Implement the terminal panel's presentation in `crates/vulcan-ui/src/panels/terminal_view.rs`, rendering fixed sample output, with no process attached
- [X] T095 [P] [US5] Implement the problems and performance dock views in `crates/vulcan-ui/src/panels/diagnostics_view.rs`
- [X] T096 [US5] Implement the tweakable properties in `crates/vulcan-ui/src/props.rs`: density, tool side, performance readout, completion style, inlay visibility, remote banner
- [X] T097 [US5] Implement the single `vkpulse` animation in `crates/vulcan-ui/src/motion.rs`, and no other transitions, matching the prototype
- [X] T098 [US5] Route every control to a no-op command sink in `crates/vulcan-ui/src/commands.rs`, so controls respond visually and perform no work, per FR-031
- [X] T099 [US5] Record every state reached that the prototype does not depict as a prototype extension, per FR-033
- [X] T100 [US5] Add the build check that fails when an unrecorded prototype extension is present, closing FR-016 and FR-033
- [X] T101 [US5] Capture the fidelity reference from the shell and confirm the comparison reports no difference
- [ ] T102 [US5] Measure the shell against every budget **it can exercise** on the authoritative
      runner, confirming the gate has a real subject. Seven of the eighteen declared metrics are
      reachable from a shell with no language server, index or search: cold start, longest
      UI-thread task, keystroke to paint, the three resident memory figures, and idle processor.
      The rest are deferred to the features that create them (T118)
- [ ] T118 Move the budgets the shell cannot exercise to the features that own them:
      `CompletionPopup` and `DiagnosticsAfterPause` to the language server feature,
      `FuzzyFileOpen` to the file finder, `ProjectTextSearch` to search, and
      `ScrollTickToPaint` to whichever feature makes the editor scroll. Each is unmeasurable
      until its feature exists, and a budget nothing can measure is a budget nothing enforces
- [X] T112 [US5] Make the shell respond to input. It renders every state but handles no event:
      there is no `on_click`, no key binding, and props are fixed at construction, so
      quickstart.md's "Prove it responds" section cannot be performed. Covers the rail, the
      collapse controls, editor tabs, the palette opening, filtering and closing, and the run
      configuration dropdown
- [ ] T113 [US5] Make the tool window and dock resizable by dragging their edge, within the
      widths the manifest declares
- [X] T114 [US5] Switch density, tool side and performance readout at runtime rather than only
      at construction
- [X] T115 [US2] Inject round-trip latency for the 0, 10, 30 and 80 ms profiles. `round_trip_ms`
      is recorded in every measurement but nothing applies it, so a completion-popup figure is a
      local measurement wearing a latency label
- [X] T116 [US2] Prove the constrained runner constrains: `cpu.max` and `memory.max` applied and
      observed, not requested. The code has only ever been seen to refuse
- [X] T117 [US2] Record real frame timings. `KeystrokeToPaint`, `ScrollTickToPaint`,
      `HighlightUpdate` and `LongestUiThreadTask` are emittable but never populated, because
      nothing calls `Instrument::record` from the render or input path. Depends on T112

**Checkpoint**: The approved interface exists, responds, and is measured. Every later feature
gives its controls something to do.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T072 [P] Measure total suite runtime and bring it under fifteen minutes per SC-007, recording the measurement in `tools/README.md`
- [X] T072a [P] Record in `tools/README.md` that the capture environment renders and composites in software, so it measures appearance only and is never a source of budget figures
- [X] T073 [P] Write `tools/README.md` covering all four gates, their exit codes, and how to run each locally
- [X] T074 [P] Create `reports/mockups-discrepancies/` with the document template from contracts/file-formats.md
- [X] T075 Record the first discrepancy in `reports/mockups-discrepancies/`. Retired 2026-09-19: the
      prototype was updated to pin JetBrains Mono, so the product and the prototype now agree and
      there is no discrepancy to record. The directory and its template stand; the triage gate
      passes over an empty set
- [X] T076 [P] Document the budget exception path in `tools/README.md`, stating that raising a budget to match a measured value is not an exception
- [X] T077 Confirm every gate exits `1` rather than `0` when a prerequisite is missing, per the quickstart's first-time contributor scenario
- [X] T103 [P] Run the visual comparison twenty consecutive times and record the rate of failures unrelated to any change, per SC-006
- [X] T104 [P] Extend the boundary gate to refuse source in any language for which no boundary rule is defined, closing FR-003
- [X] T106 Add `--screenshot <path>` to `tools/shell-preview/src/main.rs`, rendering the shell under the headless session and writing a lossless capture with no human watching
- [X] T107 Capture the feature's verification screenshot to `reports/screenshots/F000-engineering-baseline.png`, and one per notable state as `F000-engineering-baseline-<state>.png`, covering at minimum the default composition, the palette open, and each density profile
- [X] T108 [P] Add a `discrepancies` subcommand to `tools/gate-fidelity/src/main.rs`: every
      document under `reports/mockups-discrepancies/` MUST carry a Status of `Accepted` or
      `BacklogEntry`, and a `BacklogEntry` MUST name an identity present in
      `specs/features-map.md`. `Untriaged` fails with exit 2, naming the document (FR-025)
- [X] T109 [P] Extend that subcommand to fail when a design document under
      `docs/system-designs/` restates a discrepancy's body instead of linking to it (FR-027)
- [X] T111 Re-reconcile the shell against the updated prototype: the `mockups/` tree was
      replaced after the shell was built, so re-extract tokens, re-run the off-token lint,
      re-capture the fidelity reference, and correct every region whose geometry, content or
      composition the new prototype changed. The reference is stale by definition until this
      is done, and `compare` must report that rather than a regression
- [X] T105 [P] Reconcile vocabulary across spec, design and tasks: a prototype extension is a state the mock omits, a discrepancy is something we cannot match; drop the word deviation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)** blocks everything
- **Foundational (Phase 2)** blocks all user stories
- **US1, US2, US3** are independent of each other once Phase 2 is complete
- **US4** is independent of all three and could run at any point after Phase 1
- **Polish (Phase 7)** requires the gates it documents

### User Story Dependencies

- **US1 (P1)**: Foundational only
- **US2 (P1)**: Foundational only
- **US3 (P2)**: Foundational only. T059 is the first real GPUI work in the repository and carries the most unknown effort of any task here
- **US4 (P3)**: nothing — already delivered
- **US5 (P1)**: Foundational only. It is the largest phase and the subject the budget and fidelity gates measure, so US2 and US3 can be built against it but are not blocked by it

### Within Each User Story

Tests precede implementation, per Principle V. Domain types precede use cases, use cases precede adapters, adapters precede the binary, and the binary precedes build wiring.

### Parallel Opportunities

- T003 through T007: five crate skeletons
- T013 through T017: five port definitions
- All test tasks within a story, since they touch separate files
- US1, US2 and US3 can proceed concurrently once Phase 2 completes
- T041 and T042: the two runner adapters

---

## Parallel Example: User Story 1

```text
# Tests first, together:
T022  domain tests for LayerRule
T023  use-case tests for CheckBoundaries
T024  contract test for WorkspaceGraphPort

# Then implementation in dependency order:
T025 -> T026 -> T027 -> T028 -> T029 -> T030 -> T031 -> T032
```

---

## Implementation Strategy

**MVP is User Story 1 plus User Story 5.** Gate 1 makes the boundary rule mechanical; the shell makes the other gates meaningful. Together they are the smallest increment where a measurement means something.

**Previously stated, still true:** Gate 1 is the rule every later feature is written under, and
it is the cheapest of the four to build because crate-per-layer means the compiler does most
of the work. Shipping it first means F001 onward is written under an enforced boundary rather
than a remembered one.

**Then US2**, because the budgets are the product's central claim and every week they go
unmeasured buries a regression deeper.

**Then US3**, which is the largest and least predictable phase: T059 is the first GPUI code in
the repository, and its effort is the least known quantity in this feature. Treat a surprise
there as information about the framework choice, not as a task to push through.

**US4 is already done**; its three tasks are verification and documentation.

**Incremental delivery**: each story is independently mergeable and independently useful. A
gate that exists enforces its rule from that day forward, whether or not the others follow.
