# Feature Specification: Engineering Baseline

**Feature Branch**: `001-engineering-baseline`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "F000" — resolved from `specs/features-map.md`, feature F000 `engineering-baseline`

## User Scenarios & Testing *(mandatory)*

The people served by this feature are the engineers building Vulcan and the designer and
stakeholders who signed off the interface. It delivers the automated checks that the
project constitution requires but that nothing currently performs, so that the rules
governing every later feature are enforced by the build rather than by memory.

### User Story 1 - Architectural violations are caught before merge (Priority: P1)

An engineer writes code that reaches across an architectural boundary, for example
business logic that imports a database client directly. They open a change for review. The
build fails and names the offending file and the boundary it crossed, before a reviewer
spends attention on it and long before the coupling spreads.

**Why this priority**: The project's boundaries are cheap to hold from the first commit and
expensive to reinstate later. Every other feature is written on top of this rule, so a week
of unchecked commits is a week of coupling to unpick. It is also the check that is least
likely to be performed reliably by a human reviewer reading a diff.

**Independent Test**: Introduce a deliberate boundary violation on a scratch change and
confirm the build fails and names the file and rule. Remove it and confirm the build
passes. Delivers value on its own: the rule is enforced from that point onward regardless
of whether any other story ships.

**Acceptance Scenarios**:

1. **Given** a change where inner-layer code imports an outer-layer module, **When** the
   build runs, **Then** it fails and reports the file, the line, and the boundary crossed.
2. **Given** a change that respects every boundary, **When** the build runs, **Then** the
   boundary check passes and reports no findings.
3. **Given** a new language is introduced to the repository, **When** no boundary rule
   exists for it, **Then** the build fails with a message stating that a rule must be added
   before code in that language can merge.

---

### User Story 2 - Resource budgets are measured on representative hardware (Priority: P1)

An engineer makes a change that increases memory use or slows startup. The build measures
the product against the baseline machine the product targets, reports every budget metric,
and fails when a budget is exceeded. The engineer learns this from the build rather than
from a user on an 8 GB laptop months later.

**Why this priority**: The product's central promise is that it runs well on modest
hardware. A promise that is never measured is not a promise. Measuring on developer
hardware hides exactly the regressions this feature exists to catch, and each unmeasured
week buries the cause deeper in history.

**Independent Test**: Run the measurement suite against any placeholder application and
confirm it reports every budget metric with a number, on a machine constrained to the
baseline specification, and that an artificially inflated metric fails the build.

**Acceptance Scenarios**:

1. **Given** a measurement run, **When** it completes, **Then** it reports every budget the
   constitution defines: keystroke to paint, scroll tick to paint, longest interface-thread
   task, highlight update, cold start, fuzzy file open, project text search, completion,
   diagnostics, idle memory, typical memory, peak memory, and idle processor use.
2. **Given** an environment that cannot enforce the baseline constraints, **When** a
   measurement run starts, **Then** it refuses to run and says so, rather than reporting
   numbers gathered on unconstrained hardware.
3. **Given** a change that pushes a metric past its budget, **When** the build runs, **Then**
   it fails and names the metric, the budget, and the measured value.
4. **Given** a change that worsens a metric by more than ten percent while remaining under
   budget, **When** the build runs, **Then** it reports the regression for explanation
   without failing.
5. **Given** a latency-sensitive measurement, **When** it runs, **Then** it is repeated under
   simulated round-trip delays of zero, ten, thirty and eighty milliseconds, and reports each
   separately, with eighty milliseconds being the reference against which the completion
   budget is stated.

---

### User Story 3 - The signed-off interface is enforced, not remembered (Priority: P2)

An engineer implements part of the interface. The build compares what was produced against
the signed-off prototype, rejects any colour, spacing, radius or shadow that is not one of
the prototype's own values, and reports a visual difference when the result no longer
matches. Where the prototype is silent, the engineer records what they designed instead and
sends it for sign-off rather than inventing a value quietly.

**Why this priority**: The prototype is a contract with people who are not in the code
review. It is never broken by a decision; it is broken by many small approximations, each
defensible alone. This story makes those approximations visible while they are still cheap.
It ranks below the first two because it protects the product's appearance rather than its
architecture or its ability to run at all.

**Independent Test**: Run the extraction over the prototype and confirm every design value
it defines is captured. Reproduce the prototype composition, confirm the comparison reports
no difference, then change one spacing value and confirm the build fails.

**Acceptance Scenarios**:

1. **Given** the prototype file, **When** extraction runs, **Then** every colour, typography,
   spacing, radius and shadow value it defines is captured, and re-running produces an
   identical result.
2. **Given** a change introducing a design value that is not one of the extracted values,
   **When** the build runs, **Then** it fails and names the value and the file.
3. **Given** the static reproduction of the prototype, **When** it is compared against the
   reference rendering inside the pinned rendering environment, **Then** any pixel difference
   fails the build and produces a visual report showing what moved.
4. **Given** a state the prototype does not depict, such as an empty or error condition,
   **When** it is designed, **Then** it is recorded as a prototype extension with its
   sign-off status, and it uses only extracted values.
5. **Given** the prototype's own font files, **When** the reproduction is rendered, **Then**
   it uses those fonts from within the repository and requires no network access to appear
   correct.

---

### User Story 4 - Features cannot be started out of order (Priority: P3)

An engineer asks to specify a feature whose prerequisites are unfinished. The tooling
refuses, names what is blocking it, and creates nothing.

**Why this priority**: Lowest of the four because it is already delivered, by the
`featuremap` extension installed in this project. It is specified here so that the feature's
scope is recorded honestly rather than appearing in the backlog as work that was never
described.

**Independent Test**: Request a feature with an unfinished dependency and confirm the refusal
names the blocker and creates no specification directory.

**Acceptance Scenarios**:

1. **Given** a feature whose dependencies are incomplete, **When** it is requested, **Then**
   the request is refused, the blocking features are named, and nothing is created.
2. **Given** a backlog that contradicts itself, such as a completed feature containing
   unfinished parts, **When** any sequencing question is asked, **Then** the contradiction is
   reported and no answer is given until it is fixed.

---

### User Story 5 - The interface exists and responds (Priority: P1)

An engineer starts Vulcan and sees the interface the designer and stakeholders approved:
toolbar, rail, tool windows, editor surface with tabs and breadcrumbs, dock, status bar. Panels
resize and collapse, tabs switch, the command palette opens and filters, dialogs open, and the
density and layout settings take effect. Nothing behind the controls is real yet, and the
interface does not pretend otherwise.

**Why this priority**: The resource budgets are the product's central claim, and they cannot be
measured against a placeholder. Keystroke-to-paint, idle memory and longest interface-thread
task are properties of a real interface or they are properties of nothing. Building it first
also tests the rendering framework while the cost of being wrong is one feature rather than
twenty.

**Independent Test**: Start the application, compare it against the prototype and see no pixel
difference, then resize a panel, switch a tab, open and filter the palette, and change density,
confirming each responds within the frame budget.

**Acceptance Scenarios**:

1. **Given** the application is running, **When** it is compared against the prototype at the
   prototype's viewport, **Then** no pixel differs.
2. **Given** a tool window, **When** its edge is dragged or its collapse control used, **Then**
   it resizes or collapses within the frame budget and the layout remains within the
   prototype's declared dimensions.
3. **Given** the command palette, **When** it is opened and text is typed, **Then** it appears,
   filters its list, and closes on dismissal, all within the frame budget.
4. **Given** the density property, **When** it is changed between compact, default and roomy,
   **Then** every affected dimension changes to the value the prototype's manifest declares.
5. **Given** any control, **When** it is activated, **Then** it responds visually and performs
   no work behind it, because the behaviour belongs to the feature that owns it.
6. **Given** a state the prototype does not depict, **When** it is reached, **Then** it is a
   recorded prototype extension rather than an invented one.

---

### Edge Cases

- **The prototype ships without its typefaces.** The exported prototype rewrote its font
  references to local files and omitted the files themselves, so it currently renders with
  substitutes. Every fidelity comparison inherits whichever typefaces are present when the
  reference is captured, which is why capture waits for them.
- **Code type has no pinned typeface.** The prototype specifies a system monospace stack, so
  the same code renders differently on each operating system. Exact comparison cannot hold
  across platforms while the typeface is whatever the machine happens to provide.
- **The prototype changes after sign-off.** Extracted values shift, and every comparison
  reports a difference at once. The system must make clear that the prototype moved rather
  than that the product regressed.
- **Rendering differs between machines.** Anti-aliasing and font rasterisation vary across
  operating systems. Exact comparison is only meaningful inside the single pinned rendering
  environment; a comparison run anywhere else will differ for reasons unrelated to the change.
- **The build environment cannot apply the baseline constraints.** Processor and memory
  limits that are requested but not enforced produce measurements from a far larger machine
  while appearing to succeed.
- **A measurement is unstable.** Timing measurements vary between runs, so a threshold set
  too tightly fails at random and trains people to re-run the build until it passes.
- **The prototype defines no value for something that is needed.** The prototype composes one
  window size, one theme, and only the states it happens to draw.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The build MUST fail when code in an inner layer depends on an outer layer, and
  MUST report the file, the line, and the boundary crossed.
- **FR-002**: The boundary check MUST run automatically on every change, without an engineer
  choosing to invoke it.
- **FR-003**: The system MUST refuse to merge code written in a language for which no
  boundary rule has been defined.
- **FR-004**: Measurements MUST be taken on a machine constrained to six processor cores and
  eight gigabytes of memory, with those limits enforced rather than merely requested.
- **FR-005**: The system MUST refuse to report measurements when the constraints could not be
  enforced, rather than reporting unconstrained numbers.
- **FR-006**: The system MUST measure and report every budget the constitution defines,
  currently thirteen, and MUST fail rather than omit a metric it cannot measure.
- **FR-007**: The build MUST fail when any measured value exceeds its budget, naming the
  metric, the budget, and the measured value.
- **FR-008**: The system MUST report any metric that worsens by more than ten percent against
  the previous accepted measurement, whether or not it remains within budget.
- **FR-009**: The system MUST repeat latency-sensitive measurements under simulated
  round-trip delays of zero, ten, thirty and eighty milliseconds and report each separately.
- **FR-010**: The system MUST extract every design value the prototype defines, and produce
  an identical result when run again against an unchanged prototype.
- **FR-011**: The build MUST fail when a design value appears that is not one of the extracted
  values, naming the value and the file.
- **FR-012**: The system MUST compare rendered interface work against a stored reference of
  the prototype and fail the build when the difference exceeds the agreed threshold.
- **FR-013**: A failed comparison MUST produce a visual report showing what changed, not only
  a numeric difference.
- **FR-014**: The fonts, icons and images the prototype's appearance depends on MUST be stored
  in the repository, and the reproduction MUST require no network access to render correctly.
- **FR-015**: The reproduction of the prototype composition MUST use only extracted design
  values, with no literal colours, spacing, radii or shadows.
- **FR-016**: Any interface state the prototype does not depict MUST be recorded as a
  prototype extension, with its sign-off status, before it is implemented.
- **FR-017**: The system MUST refuse to begin specifying a feature whose declared
  prerequisites are unfinished, naming what blocks it and creating nothing.
- **FR-018**: The system MUST refuse to answer sequencing questions while the backlog
  contradicts itself, and MUST report the contradiction.
- **FR-019**: Every check in this feature MUST be runnable locally by an engineer, producing
  the same verdict it produces in the build.
- **FR-020**: The fidelity reference MUST NOT be captured until the prototype's typefaces are
  present in the repository, and MUST record which typefaces were present at capture, so that
  a later comparison failure caused by a font change is distinguishable from one caused by a
  code change.
- **FR-021**: The system MUST treat the asset manifest at `mockups/assets.md` as part of the
  appearance contract alongside the prototype file. Extraction MUST capture the design values
  the manifest documents outside the prototype's own custom properties, including the semantic
  state colours and the syntax palette, because a value documented only in the manifest is
  still a value the product is required to use rather than invent.
- **FR-022**: Budget failures MUST block a change from merging, from the first measurement
  onward. A change may proceed over a budget failure only by recording an exception naming
  the metric, the reason, and the simpler alternative rejected, under the same exception rule
  the constitution applies to complexity. Raising a budget to match a measured value is not
  an exception and MUST NOT be used in place of one.
- **FR-023**: The visual comparison MUST treat any pixel difference as a failure, with no
  tolerance threshold.
- **FR-024**: All visual comparisons MUST run in a single pinned rendering environment, so
  that exact comparison is stable. That environment MUST be runnable by an engineer on their
  own machine, so the local verdict and the build verdict agree; a comparison performed
  outside it is advisory and MUST NOT be treated as authoritative.
- **FR-025**: When the product cannot match the prototype exactly, the discrepancy MUST be
  recorded under `reports/mockups-discrepancies/` before the work is accepted, as one document
  per discrepancy stating what was being matched, why it cannot be matched, and the
  alternatives available. A discrepancy is distinct from a prototype extension, which is a
  state the prototype does not depict and is recorded in the feature's design document.
- **FR-026**: Each recorded discrepancy MUST be triaged individually into either a new backlog
  entry or an accepted difference, and the document MUST record which was chosen. Discrepancies
  MUST NOT accumulate untriaged, and MUST NOT be resolved by loosening the comparison.
- **FR-027**: A feature's design document MUST reference the discrepancy documents that apply
  to it by path, and MUST NOT restate their contents, so that the discrepancy record has one
  home rather than two that drift.

- **FR-028**: The interface MUST render every region the prototype composes: toolbar, remote
  banner, rail, tool windows, tab strip, breadcrumbs, editor surface, dock with its tab strip,
  status bar, command palette, and the run configuration and language pack dialogs.
- **FR-029**: Regions MUST use the fixed dimensions the prototype's manifest declares, and MUST
  change with the density property between its compact, default and roomy values.
- **FR-030**: Panels MUST resize and collapse, tabs MUST switch, the command palette MUST open,
  filter and close, and dialogs MUST open and dismiss, each within the frame budget in the
  constitution.
- **FR-031**: Controls MUST NOT perform work behind them in this feature. A control that appears
  to act but does nothing is acceptable; one that half-acts is not, because a partial behaviour
  cannot be distinguished from a defect by the feature that later owns it.
- **FR-032**: The interface MUST use only extracted design values, and MUST NOT fetch anything
  from a network to render correctly.
- **FR-033**: Any interface state reached that the prototype does not depict MUST be recorded as
  a prototype extension with its sign-off status before it is implemented, and a build MUST fail
  when an unrecorded extension is present.

### Key Entities

- **Design value**: A single named appearance decision taken from the prototype — a colour, a
  typeface, a spacing step, a corner radius, a shadow. Has a name and a value, and is the
  only permitted source of appearance in the product.
- **Budget measurement**: One metric measured in one run on the constrained machine. Has a
  metric name, a measured value, the budget it was compared against, the simulated network
  delay in force, and a verdict.
- **Fidelity reference**: A stored rendering of the prototype against which later work is
  compared. Has a capture date, the fonts present at capture, the window size used, and the
  prototype version it was taken from.
- **Prototype extension**: An interface state designed because the prototype does not depict
  it. Has a description, the design values it uses, and a sign-off status.
- **Backlog entry**: A feature in the ordered backlog. Has an immutable identity, a position,
  the features it depends on, a completion state, and the specification it produced.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A change containing an architectural boundary violation is rejected without any
  human having to notice it, in one hundred percent of cases.
- **SC-002**: Every budget metric the constitution defines is reported with a number on every
  build, with no metric silently absent.
- **SC-003**: An engineer can reproduce any build verdict on their own machine, with no check
  that exists only in the build environment.
- **SC-004**: One hundred percent of design values present in the product resolve to values
  defined by the prototype.
- **SC-005**: A single-pixel spacing change made anywhere in the reproduced composition is
  detected by the comparison.
- **SC-006**: Fewer than one in twenty comparison runs fails for a reason unrelated to the
  change under test, measured across at least twenty consecutive runs.
- **SC-007**: The complete set of checks finishes within fifteen minutes, so that it can run
  on every change without engineers learning to avoid it.
- **SC-008**: Every interface state not depicted in the prototype is recorded with a sign-off
  status before it is implemented, with no unrecorded extensions in the product.

- **SC-009**: Every region the prototype composes is present in the running application, with
  none omitted.
- **SC-010**: Every interaction listed in User Story 5 completes within the frame budget on the
  baseline machine, measured by the budget gate rather than by impression.
- **SC-011**: Zero interface states exist in the application that are neither in the prototype
  nor recorded as a prototype extension.

## Assumptions

- The people served by this feature are the Vulcan engineering team, the designer who signed
  off the prototype, and the stakeholders who approved it. There are no end users at this
  stage, because the product has no released functionality.
- The self-contained prototype under `mockups/` is the signed-off appearance reference, as the project
  constitution states. It composes one window size, one theme, and only the states it draws;
  everything else is a prototype extension.
- `mockups/assets.md` is the asset and token manifest for the prototype and is part of the
  appearance contract. It names the typefaces precisely enough to recover them without the
  designer: Inter at weights 400, 500, 600 and 700, and Phosphor Icons 2.1.1 regular. The
  fidelity reference is captured after both are vendored into the repository.
- The prototype specifies a system monospace stack rather than a named typeface. Vulcan pins
  JetBrains Mono for code type instead, so that code renders identically on every platform,
  which exact comparison requires. Interface type remains Inter as the prototype specifies.
  This is a deliberate departure from the prototype and is the first entry in
  `reports/mockups-discrepancies/`.
- The manifest documents fourteen colours outside the prototype's custom properties: five
  semantic state colours and nine syntax colours derived from the neutral and accent ramps.
  These are design values despite not being tokens in the stylesheet, so extraction reads the
  manifest as well as the prototype.
- The budget values, the baseline machine of six cores and eight gigabytes at A18 Pro class
  with a 120Hz display, and the ten percent regression rule are taken from the project
  constitution and are not re-decided here. The baseline's cores are asymmetric, two
  performance and four efficiency, so the measurement environment must reproduce that
  asymmetry rather than providing six equal cores.
- The budgets originate from an architecture document that has since been deleted from the
  repository. The constitution is now their only home, which makes its accuracy load-bearing
  for this feature.
- Measurements are assumed to run in an environment where processor and memory limits can be
  enforced rather than merely requested. Where they cannot, the feature reports rather than
  estimates.
- The sequencing check described in User Story 4 is already implemented and installed in this
  project. Its specification here records existing behaviour rather than requesting new work.
- Approximately five hundred lines of sequencing tooling already exist in this repository,
  written before this specification. This feature adopts that code rather than rewriting it,
  and the design document is expected to record it as pre-existing.
- Visual comparison is exact rather than perceptual, which is only tenable because every
  comparison runs in one pinned rendering environment. Removing that environment would make
  exact comparison unusable, not merely stricter.
- Budget enforcement blocks merges from the first measurement. The exception path is expected
  to be used during early spikes, and its value is that those exceptions are visible across
  features rather than absorbed into a raised budget.
- Fidelity discrepancies live in `reports/mockups-discrepancies/`, one document per
  discrepancy, carrying what was being matched, why it cannot be, and the alternatives.
  Feature design documents reference them by path rather than restating them. Whether a
  discrepancy becomes a backlog entry or an accepted difference is decided case by case, not
  by a standing rule.
- The constitution separates the two kinds of departure as of v3.0.1: a prototype extension,
  a state the prototype does not depict, stays in the feature's design document; a
  discrepancy, something the product cannot match, is recorded under
  `reports/mockups-discrepancies/` and referenced by path. This assumption is discharged; the
  two locations no longer disagree.
- The interface built here is the product's shell, not a throwaway fixture. It is written
  before the plugin host exists, so it sits outside the plugin model and is re-hosted through
  extension points by F002. That retrofit is accepted deliberately, in exchange for measuring
  the budgets against something real from the first feature.
- Scope is presentation and local interaction only. Opening files, contacting a language server,
  running a command, and rendering real terminal output belong to the features that own them.
- The prototype composes one viewport and one theme. Resizing the window, any second theme, and
  every empty, loading or error state are prototype extensions requiring sign-off.
- The gates are demonstrable against the shell as it grows, rather than against a placeholder,
  which is the reason the shell is in this feature at all.
