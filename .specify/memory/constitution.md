<!--
SYNC IMPACT REPORT
Version change: 4.0.0 -> 4.1.0
Bump rationale: MINOR. A Principle VI budget is relaxed. Code compliant at the
old figure is compliant at the new one, so nothing is invalidated, but what is
required has changed and that is more than a clarification.

Idle processor, mean: 1% -> 2%.

The original figure was set before anything had been measured, so it was an
aspiration rather than a calibrated target. It is not achievable with the chosen
user interface framework, and the evidence is that the framework alone costs it:

- An empty GPUI window, containing one element and no product code, idles at
  1.34%, 1.19% and 1.10% across three runs.
- The whole Vulcan shell idles at 1.14% to 1.35% on the same machine, and 1.35%
  to 1.57% on an M3 Pro.
- The shell draws zero frames across an idle window, so it is not redrawing
  needlessly; the cost is GPUI's event loop, not work this product does.

The shell therefore adds nothing measurable to the framework's floor, and no
amount of work on this codebase would reach 1%.

2% rather than a rounder number: it clears the worst observation of 1.57% with
margin for machine variance, and still fails if anything this product adds on
top of the framework doubles. A budget set far above the measurement stops being
a check, which is why 5% was considered and rejected.

This figure is a property of GPUI rather than of Vulcan. If the framework is
upgraded or replaced, measure again: tools/idle-probe exists to ask exactly this
question and records its own history.

Modified principles: VI, one budget value.
Added sections: none. Removed sections: none. Deferred items: none.
-->

<!--
SYNC IMPACT REPORT
Version change: 3.0.1 -> 4.0.0
Bump rationale: MAJOR. Two constitutions existed in parallel on separate branches
with no shared history: a product-focused one on the original trunk, and an
architecture-focused one written during F000. Neither was a draft of the other.
This merges them, so the document governs both how the product must behave and
how it must be built.

Carried forward from the original trunk, renumbered and otherwise unchanged:
- X. The Keystroke Path Is Sacred
- XI. Snapshots, Cancellation, Versioned Results
- XII. No Feature Waits on a Server or an Index
- XIII. Local and Remote Are One Implementation
- XIV. Extensions Are Data First, Sandboxed Code Second

Reconciled rather than duplicated, because each had a counterpart:
- Budgets Are CI Gates -> VI. Runs on the Baseline Machine
- Test-First -> V. Test at the Boundary
- The Approved Mock Governs the Interface -> VIII. Prototype Fidelity
- Bounded Footprint on Modest Hardware -> VI. Runs on the Baseline Machine

Corrected: the baseline machine is 6 CPU cores and 8 GB, not the 5 CPUs and
12 GB the original trunk stated. Every Principle VI budget is measured against
the corrected figure. One consequential reference inside Principle XIV was
updated from 12 GB to 8 GB to match.

Modified principles: none of I-IX changed.
Added sections: principles X through XIV.
Removed sections: none.
Deferred items: none.
-->

<!--
SYNC IMPACT REPORT
Version change: 3.0.0 -> 3.0.1
Bump rationale: PATCH. Principle VIII's requirement is unchanged: departures from the
signed-off prototype must be recorded and returned to the designer. What changes is where
one of them lives. Discrepancies now have a single home at `reports/mockups-discrepancies/`
rather than inside each feature's design document, which resolves a contradiction with the
F000 specification that had five requirements built on it.

The amendment also separates two concepts that had been drifting together in the
specification and task artifacts: a prototype extension is a state the mock omits, and a
discrepancy is something the product cannot match. The first stays in the design document;
the second moves to its own directory, referenced by path.

Modified principles:
- VIII. Prototype Fidelity: departure recording split by kind and relocated.
- I through VII, IX: unchanged.

Added sections: none

Removed sections: none

Deferred items: none.
-->

<!--
SYNC IMPACT REPORT
Version change: 2.3.1 -> 3.0.0
Bump rationale: MAJOR. Principle VI's resource budgets were replaced wholesale with a
stricter set, and work that satisfied the previous budgets can fail the new ones. Cold
start tightens from 3s to 300ms, idle memory from 600MB to 400MB, and the longest
permitted UI-thread task from 50ms to 8ms. The completion budget is unchanged at 250ms p95
against an 80ms reference round trip.
Several new budgets were added that had no previous equivalent.

Modified principles:
- VI. Runs on the Baseline Machine: budget table replaced; the baseline machine is now
  named as a 6-core, 8 GB reference of A18 Pro class, and the frame budget is stated at
  120Hz rather than 60Hz.
- I through V, VII through IX: unchanged.

Added sections: none

Removed sections: none

Deferred items: the budgets originate from an architecture document that has since been
deleted from the repository. They are transcribed here so the numbers survive their
source, and this constitution is now their only home.
-->

<!--
SYNC IMPACT REPORT
Version change: 2.3.0 -> 2.3.1
Bump rationale: PATCH. Two clarifications that change no requirement. The feature map's
location moved from `docs/feature-map.md` to `specs/features-map.md`, alongside the
specification directories it sequences, and gate 9's enforcement is now automated rather
than deferred, so the text describing it as manual is corrected.

Modified principles: none. Principle IX's requirements are unchanged; only the path it
names and the description of how the gate is enforced were updated.

Added sections: none

Removed sections: none

Deferred items: none.
-->

# Vulcan Constitution

## Core Principles

### I. Domain Independence (NON-NEGOTIABLE)

Dependencies MUST point inward only: adapters depend on the application layer, the
application layer depends on ports and the domain, and the domain depends on nothing
outside itself. Code under `domain/` MUST NOT import web frameworks, ORMs, HTTP or
queue clients, vendor SDKs, serialization annotations, or dependency-injection
containers. Code under `application/` MUST NOT import concrete adapter modules; it
may import only domain types and port interfaces.

Rationale: the direction of dependency is the whole architecture. Once an entity
imports an ORM base class or a use case imports a framework request type, the
business rules can no longer be read, tested, or moved without the infrastructure
that surrounds them. This rule is stated as an import constraint precisely so it can
be enforced mechanically rather than argued case by case in review.

### II. Every Side Effect Is a Port

Every interaction with the outside world MUST be expressed as an outbound port
interface declared in the application layer and implemented by an adapter. This
includes persistence, HTTP and RPC calls to other systems, message publication,
file and blob access, and the ambient dependencies that make tests nondeterministic:
clock, random, UUID generation, and environment configuration. Ports MUST be named
for the capability they provide, never for the technology behind them
(`OrderRepositoryPort`, not `PostgresPort`).

Rationale: a side effect that has no port cannot be faked, which means the use case
that performs it cannot be unit tested and the technology behind it cannot be
replaced. Naming ports after capabilities keeps the vendor swappable; naming them
after vendors re-couples the core to the infrastructure through the interface itself.

### III. Use Cases Own Orchestration

Each use case MUST expose a single behavior with an explicit input type and an
explicit output type, both of which are plain data owned by the application layer.
Inbound adapters (HTTP handlers, CLI commands, queue consumers, schedulers) MUST
translate protocol payloads into use-case input and translate use-case output and
errors back into protocol form, and MUST NOT contain business branching, persistence
calls, or cross-service coordination. Use cases MUST NOT receive framework request,
response, session, or job-metadata objects, and MUST NOT return raw database rows or
vendor response objects. Infrastructure errors MUST be translated into domain or
application errors at the adapter boundary.

Rationale: when orchestration lives in a controller, the same behavior cannot be
reached from a second transport without duplication, and the business rules become
reachable only by booting the framework. Explicit input and output types are what
make a use case addressable from HTTP, CLI, a worker, and a test alike.

### IV. Explicit Composition Root

Concrete adapters MUST be bound to use cases in a single composition root per
deployable unit, and that wiring MUST be readable as ordinary code. Service locators,
ambient global singletons, and runtime container lookups from inside domain or
application code are prohibited. Use cases MUST receive their ports through
constructor or function parameters.

Rationale: the composition root is the one place where the architecture's seams are
visible. Spreading wiring across dozens of decorated classes hides which
implementation is actually live, makes substitution in tests unreliable, and lets
infrastructure concerns leak back into the core through the container.

### V. Test at the Boundary (NON-NEGOTIABLE)

Tests MUST be written against the same boundaries the architecture defines:

- Domain rules are tested as pure functions and values, with no mocks and no
  framework bootstrap.
- Use cases are tested with in-memory fakes for their outbound ports, asserting
  business outcomes.
- Each outbound port has one shared contract test suite, executed against every
  adapter implementing that port, including the in-memory fake.
- Adapters are integration tested against real infrastructure for serialization,
  schema and query behavior, timeouts, and retries.
- User-facing capabilities are covered end to end through a real inbound adapter.

In the common vocabulary, the unit tier is the domain and use-case tests, the
integration tier is the adapter tests plus the port contract suites, and the
end-to-end tier is a real inbound adapter driving real outbound adapters.

Every change that adds or modifies behavior, whether a new feature or an enhancement to
an existing one, MUST carry its tests in the same change, at every tier the change
touches:

- behavior added or changed in domain or application code requires unit tests;
- an adapter, schema, query, or external contract added or changed requires integration
  tests, including an update to the affected port contract suite;
- a user-facing capability added or changed requires at least one end-to-end test
  covering its primary success path, with failure and edge behavior pushed down to the
  cheaper tiers.

Omitting a tier is permitted only when that tier does not apply to the change, and the
pull request MUST name the omitted tier and the reason. "Tests to follow" is not a
reason, and a follow-up ticket does not substitute for the tests.

Modifying code that has no tests MUST begin by adding characterization tests that pin
current observable behavior, before the modification is made.

Domain and use-case code MUST be developed test-first: a failing test precedes the
implementation. Tests MUST assert observable behavior, not internal call sequences,
except where a port interaction is itself the specified outcome.

The unit tier MUST complete in under 2 minutes on the baseline machine defined in
Architectural Constraints, and the full suite MUST be runnable there. A suite that only
runs on CI hardware is treated as a budget breach under Principle VI.

Rationale: the contract-test rule is what keeps the in-memory fake honest; without it
use-case tests drift into passing against a fake that no real adapter matches. TDD is
mandated only for the two layers where the cost of a wrong rule is highest and the cost
of writing a test first is lowest, since neither layer requires infrastructure to run.
Tests are required in the same change because tests written afterwards are written
against the implementation instead of the requirement, inherit its misreadings, and in
practice frequently never arrive. End-to-end coverage is required per user-facing
capability but deliberately limited to the primary path, because that tier is the
slowest and most failure-prone: one test per capability catches wiring and composition
errors that no lower tier can see, while depth at that tier buys flakiness rather than
confidence. Characterization tests come first when touching untested code because
without them a refactor cannot be distinguished from a behavior change.

### VI. Runs on the Baseline Machine

Every desktop deliverable MUST remain fully usable on the baseline machine defined in
Architectural Constraints, and MUST be measured there rather than on developer
hardware. The resource budgets in that section are limits, not targets: a change that
pushes any metric past its budget MUST NOT merge until it is brought back under, or an
exception is recorded under the complexity rule in Governance.

Hardware acceleration, background indexing, local model inference, prefetching, and
similar accelerants MUST be optional. Each MUST have a functional degraded path that
keeps the product usable when the capability is absent, disabled, or starved, and the
degraded path MUST be exercised by tests rather than assumed.

Work that would occupy the UI thread for longer than one frame MUST be
moved off it and reached through an asynchronous outbound port, so that the boundary
that makes the work testable is also the boundary that keeps the interface responsive.
Caches and queues MUST have explicit bounds and eviction rules; unbounded growth is
treated as a defect regardless of whether it manifests on a developer machine.

Rationale: 8 GB is the whole machine's budget, not the application's. The operating
system, a browser, and a chat client are resident before the product starts, so an
application that measures fine in isolation can still make the machine unusable. A 64 GB
workstation hides every regression of this class until a user finds it, and by then the
cause is dozens of commits back. Fixed budgets measured under baseline constraints turn
"it feels fine" into a number that CI can fail on, which is the only form of performance
requirement that survives delivery pressure.

### VII. Design Before Code

Every feature, and every enhancement that changes a boundary, MUST have a committed
architecture document under `docs/system-designs/` before its first implementation
commit. The document MUST cover both levels: the high-level design, which states how the
feature sits in the system, and the low-level design, which states the domain model,
ports, use cases, and adapters the implementation will create. Required contents are
listed in Development Workflow and Quality Gates.

Enhancements to an existing feature MUST update that feature's existing document rather
than add a second one. Documents are superseded in place and annotated, never silently
replaced or deleted, so that the reasoning behind the current shape stays recoverable.

When implementation reveals that the design is wrong, work MUST stop, the document MUST
be corrected in the same change that departs from it, and the departure MUST be visible
in review. A design document that no longer matches the code is worse than none, because
it is trusted.

Rationale: the boundaries this constitution mandates in Principles I through IV are
cheap to draw before code exists and expensive to move afterwards. Writing down the
ports and the use cases first is the only point at which a reviewer can object to a
boundary without asking for a rewrite. The two levels are both required because they
fail differently: a high-level design alone hides the coupling that appears when
signatures are written, and a low-level design alone hides the system-level consequence
of the feature. The stop-and-correct rule exists because the common failure is not
designing badly, it is designing well and then diverging silently under delivery
pressure, which converts the document into misinformation.

### VIII. Prototype Fidelity

`mockups/Vulcan-IDE.html` is the signed-off reference for the product's appearance and
interaction. Where an implementation and the prototype disagree on how the product
looks or behaves at the surface, the prototype wins and the implementation changes.

The prototype is authoritative for layout, spacing, typography, color, iconography, the
component states it draws, and the interaction affordances it demonstrates. It is not
authoritative for anything behind the surface: architecture, module structure, data
shapes, and technology remain governed by Principles I through IV and by `plan.md`. A
prototype is a picture of the outside of the system and MUST NOT be read as instructions
for the inside of it.

The file is under change control. It MUST NOT be edited to accommodate an implementation
shortcut. Any change requires recorded designer and stakeholder approval, and the
superseded version MUST be retained so that what was signed off remains recoverable.

The prototype's CSS custom properties are the design system: the `--color-*`, `--font-*`,
`--space-*`, `--radius-*`, and `--shadow-*` families are the single source of design
values. Implementation MUST consume tokens extracted mechanically from the prototype and
MUST NOT hardcode literal colors, spacing, radii, or shadows. Extraction MUST be a script
that can be re-run, so that a prototype change surfaces as a token diff instead of as a
visual surprise. Every asset the prototype's appearance depends on MUST be vendored into
the repository; the shipped product MUST NOT fetch fonts, icons, or images from a network
to look correct.

The prototype does not define every state. It composes one viewport, one theme, and the
states it happens to draw. Loading, empty, error, overflow, disabled, focus, window
resizing, and any additional theme are undefined by it. Undefined cases MUST be derived
from the existing token system, recorded in the feature's design document under a
prototype extensions heading, and submitted for sign-off. Introducing a design value that
is not in the token system is prohibited.

Where fidelity conflicts with another principle, the order of precedence is fixed:

1. Accessibility floor. Contrast, focus visibility, keyboard operability, and assistive
   technology support MUST meet WCAG 2.2 AA. Where the prototype falls below the floor,
   the implementation meets the floor and the deviation returns for re-sign-off.
2. Resource budgets. Where fidelity cannot be achieved within the Principle VI budgets on
   the baseline machine, the budgets win. Degradation MUST be confined to effects such as
   shadows, transitions, and animation, and MUST NOT alter layout, type scale, or color.
3. Prototype fidelity, over any implementation convenience or individual preference.

Two kinds of departure are recorded, and they are not the same thing:

- A **prototype extension** is a state the prototype does not depict, such as an empty or
  error condition, a second theme, or a window size it does not compose. It is designed from
  the existing token system and recorded in the feature's design document, as stated above.
- A **discrepancy** is something the product cannot match, permitted by the ladder above. It
  is recorded as one document under `reports/mockups-discrepancies/`, stating what was being
  matched, why it cannot be, and the alternatives available. Each is triaged individually
  into either a backlog entry or an accepted difference, and a feature's design document
  references it by path rather than restating it.

Either kind, unrecorded, is a defect. A discrepancy left untriaged at review time is a
blocking finding, and loosening the comparison is never a resolution.

Rationale: a signed-off prototype is a contract with people who are not in the code
review, and the normal way that contract breaks is not a decision to break it but a
thousand small approximations, each defensible alone. Tokens extracted by script and
compared automatically are what make those approximations visible while they are still
cheap. The precedence ladder exists because sign-off cannot ratify an interface that
excludes users with disabilities, and because a faithful interface that misses the
resource budgets is unusable on the hardware this product targets. Naming the order in
advance prevents the question being relitigated per feature under delivery pressure.

### IX. The Feature Map Governs Sequence

`specs/features-map.md` is the ordered backlog and the only authority on what may be worked
on next. Features carry an immutable identity of the form `F<NNN>` and a position in the
file. The number identifies the feature for all time; the position states where it falls
in the build order.

Checkboxes are evidence, not intention. A subfeature box MUST NOT be checked until that
slice is merged with its tests passing. A feature box MUST NOT be checked until every one
of its subfeatures is checked, its specification and design document exist, and gates 1
through 8 are green for it. Checking a box that these conditions do not support is a
defect of the same severity as a failing gate, because every later sequencing decision
reads those boxes as fact.

Before a feature is specified, every feature it declares as a dependency MUST be fully
checked. The precondition is the declared dependencies, not every entry positioned above
it: features marked `[P]` are declared independent of one another, and serialising them
would cost schedule without reducing risk. When `/speckit-specify` is invoked without a
feature identifier, the feature to specify is the first unchecked feature in file order
whose declared dependencies are all checked.

When a precondition is unmet, work stops. The blocking feature MUST be named and no
specification directory is created. Starting a feature whose dependencies are incomplete
produces a specification written against assumptions that the missing work has not yet
validated.

Feature numbers are never renumbered and never reused. A feature inserted into the middle
of the sequence takes the next unused number and is placed at its correct position in the
file; a feature that is dropped is struck with a recorded reason rather than deleted.

Rationale: the backlog only functions as a coordination artifact if its state is
trustworthy and its identifiers are stable. Renumbering on insertion would invalidate
every reference held by specification directories, design documents, commit messages and
review threads, in exchange for the cosmetic property of numbers ascending down the page;
separating identity from position costs one sentence of convention and keeps every
reference valid forever. Binding the checkboxes to the gates rather than to a judgement
of progress is what stops the map from becoming an optimistic summary, which is the
normal way a tracking document stops being consulted.

### X. The Keystroke Path Is Sacred (NON-NEGOTIABLE)

Nothing crosses a process boundary, a network boundary, or a lock that a background thread can
hold, between a key event and the pixel it produces. The path does exactly three things: mutate
the buffer, re-shape the affected line(s), repaint the damaged rectangle.

- Keystroke → paint MUST complete within one 120 Hz frame. The frame deadline is **8.33 ms**;
  the CI gate is set at **8.0 ms p99**, preserving roughly 4% margin so that harness jitter
  cannot let a genuinely dropped frame pass. Scroll tick → paint meets the same gate.
  Where the design doc states 8 ms and 8.3 ms in different places, this is the distinction it is
  drawing without naming: 8.33 ms is physics, 8.0 ms is the threshold Vulcan enforces.
- The hot path MUST NOT perform blocking I/O, acquire a contended lock, allocate unboundedly,
  parse JSON, or await any response from a language server, debug adapter, extension, index,
  or remote agent.
- Any design that places serialization between keystroke and glyph MUST be rejected at review
  time, regardless of its other merits. This is the error that ended xi-editor and the reason
  Electron editors measurably lag native ones.
- Input arriving mid-frame is processed for the next frame. Worst-case latency is therefore
  bounded at roughly two frames by construction, not by tuning.

**Rationale:** §4.1 records the same JVM editor moving from 24.7 ms mean / 239.3 ms max to
2.9 ms mean by removing a per-keystroke document lock and an over-broad repaint — a ~20× swing
with the language held constant. Hot-path discipline is the single largest lever available, and
it is an architectural property that cannot be recovered by optimization later. (§5.1, §10, §11,
§19.1)

### XI. Snapshots, Cancellation, Versioned Results

The UI thread owns the truth. Every other consumer — syntax, indexer, diff, search, LSP/DAP
clients, extensions, the remote agent — works on immutable copies.

- Buffers and other UI-visible state MUST be mutated only on the UI thread.
- Background work MUST receive an immutable snapshot plus a monotonic version number, never a
  live reference.
- Every background task MUST be cancellable and MUST be cancelled when an edit invalidates it.
  In-flight completion and hover requests are cancelled on the next keystroke, and cancellation
  MUST propagate across the remote channel to the server.
- Every result MUST carry the version it was computed against. A result for an older version is
  either mapped forward through anchors or discarded — never merged blind.
- Positions that must survive edits (breakpoints, diagnostics, folds, bookmarks) MUST be stored
  as anchors, never as raw offsets.
- Results crossing back to the UI thread MUST be already-shaped and small. Decoding, merging,
  ranking, and position conversion happen off the UI thread.

**Rationale:** Cancellation-on-edit is the mechanism that prevents "the editor froze because
completion was still computing against the old text." Anchors are why diagnostics do not drift
onto the wrong line after an insertion above. A documented Neovim bug froze the UI for ~10 s on
a ~2,000-item references response solely because the transformation ran on the main thread.
(§9.1, §10, §14.3)

### XII. No Feature Waits on a Server or an Index

Language intelligence is advisory. The editor MUST remain fully usable at any server latency,
including infinite, and at any index completeness, including zero.

- A feature MUST NOT be disabled while indexing. Where an index-backed answer is unavailable,
  the feature MUST degrade to a cheaper source — grammar-derived tags for symbols, a scanning
  text search — rather than disappear. IntelliJ's "dumb mode" is the failure case being
  designed against.
- Stale results MUST be shown, visibly labelled as stale, rather than withheld pending a fresh
  answer.
- A crashed or hung server MUST NOT degrade editing. Restart uses exponential backoff; after N
  failures the IDE stops and shows a non-blocking notice. All requests have timeouts.
- Above a configured file-size threshold, the syntax layer and semantic features are switched off
  and plain editing stays fast.
- Diagnostics, diffs, and blame are recomputed on save or after a debounce. They are never on
  the keystroke path.

**Rationale:** "Snappy on any language" is a boundary-placement property, not a language
property. Tree-sitter in-process for syntax and LSP/DAP out-of-process for semantics delivers it
only if the editor genuinely never gates on the out-of-process half. (§9.5, §9.6, §13, §17.4,
§18)

### XIII. Local and Remote Are One Implementation

Remote development is not a mode bolted onto a local IDE. It is the same code, running headless.

- The remote agent MUST be the same codebase's Local variants of `Worktree`, `LspStore`,
  `DapStore`, and the indexer, minus the UI crates. A second implementation of the VFS, the
  buffer, or the indexer MUST NOT exist.
- Core components MUST be reachable through one interface with Local and Remote variants, so
  callers cannot tell which they hold.
- The keystroke path is identical in both modes: buffer, undo, and syntax highlighting stay
  local, and the network MUST NOT appear in steps 1–6 of the §11 trace. Typing measurably slower
  in remote mode is an architecture defect, not a link problem.
- The client is authoritative for text. The agent holds versioned shadow copies and applies the
  same edit transactions.
- What crosses the network MUST be Vulcan's compact binary protocol carrying already-shaped
  results — never raw JSON-RPC, which would put decoding of large payloads on the least powerful
  machine in the system. DAP and PTY streams are the deliberate exception and are forwarded
  nearly raw.
- Agent sessions MUST outlive their connection. A dropped link MUST NOT restart language servers,
  terminals, debug sessions, or the indexer, and MUST NOT lose typed text. Reconnect resumes by
  session ID and conflict-checks remote file state before replaying.

**Rationale:** VS Code Remote, JetBrains Gateway/Fleet, and Zed converged on this split
independently. Reusing the local concurrency model for remote means remote mode adds no new
correctness surface, and it is what makes a small client machine viable. (§14)

### XIV. Extensions Are Data First, Sandboxed Code Second

The extension architecture is built in milestone 1; the public API is frozen and published only
after Vulcan's own features ship through it.

- There are exactly two extension kinds: **declarative packs** (manifest plus assets — grammars,
  queries, language-server and debug-adapter definitions, themes, keymaps, snippets, run-config
  templates) and **WASM extensions** (sandboxed, executed by an in-process WASM runtime, with
  capability-based permissions declared up front in the manifest).
- Adding a language MUST cost zero host code and no rebuild.
- Extension calls MUST be asynchronous, run on the background pool, and be time-limited. A hung
  extension is killed and reported; the UI thread MUST NOT await one.
- Extensions MUST see buffer snapshots, never live buffers. Edits return as transactions the host
  applies.
- UI contributions MUST be declarative. Extensions describe panels, status-bar items, tree views,
  and commands as data; the host renders them. Extensions MUST NOT draw pixels or run on the UI
  thread.
- Per-extension memory and CPU MUST be metered and visible in the resource panel.
- Vulcan's own Java pack, default theme, and IntelliJ keymap MUST ship through this system before
  any third party sees the API.

**Rationale:** IntelliJ's in-process JVM plugins buy unmatched depth and cost the ability to
guarantee anything about latency or memory. On an 8 GB machine, isolation is the correct trade.
Dogfooding the API before publishing it is the only way to avoid guessing at its shape.
(§9.11, §17.5)

## Architectural Constraints

Every new project, service, or deployable in this repository MUST be scaffolded as
Ports and Adapters before the first feature is implemented. Retrofitting boundaries
after delivery pressure arrives is explicitly out of policy.

Modules MUST be organized feature-first, with layer folders nested inside each
feature rather than the reverse:

```text
src/
  features/
    <feature>/
      domain/
      application/
        ports/
          inbound/
          outbound/
        use-cases/
      adapters/
        inbound/
        outbound/
      composition/
```

Rationale: feature-first grouping keeps a vertical slice deletable and reviewable in
one place; layer-first grouping scatters a single change across the tree and makes
boundary violations harder to see in a diff.

Language mappings preserve the same boundaries and differ only in syntax and wiring:
TypeScript uses interfaces plus explicit factory functions; Java and Kotlin use
`domain`, `application.port.in`, `application.port.out`, `application.usecase`,
`adapter.in`, `adapter.out` packages with constructor injection; Go uses small
interfaces owned by the consuming application package with wiring in `cmd/<app>`.

Cross-feature access MUST flow through a published port or use case of the owning
feature. Direct imports of another feature's `domain/` or `adapters/` are prohibited.

Adapters MUST NOT call other adapters directly; a flow that needs two side effects
belongs in a use case that depends on both ports.

Legacy or non-conforming code MUST be migrated by vertical slice using a strangler
approach: wrap the existing behavior behind an outbound port, add characterization
tests, move orchestration into a use case, then route the old entry point through it.
Full rewrites are prohibited, because they replace verified behavior with unverified
behavior in a single step.

**Baseline machine.** All performance claims and gates refer to a machine with 6 CPU
cores, 8 GB RAM, integrated graphics with no discrete GPU, an SSD, and a 120Hz display.
The reference is hardware of A18 Pro class, whose six cores are asymmetric: two
performance cores and four efficiency cores. Work scheduled as though six equal cores were
available will not meet these budgets, so parallelism assumptions MUST be stated and
measured rather than assumed. Measurement MUST
occur on that hardware or on a VM or container constrained to it (CPU quota and memory
limit applied, not merely requested), with the application under a realistic workload
rather than an empty project.

**Resource budgets.** Values apply to the sum of all processes the application owns:

| Metric | Budget |
| --- | --- |
| Keystroke to paint | 8 ms p99 |
| Scroll tick to paint | 8 ms p99 |
| Longest task on the UI thread | 8 ms |
| Highlight update after an edit | 16 ms, off the UI thread |
| Cold start to a rendered file | 300 ms |
| Fuzzy file open across 100,000 files | 50 ms per query |
| Project text search, first results | 500 ms |
| Completion popup | 250 ms p95 end to end, at a round trip of 80 ms or less, never blocking |
| Diagnostics after a typing pause | 300 ms to 2 s, never blocking |
| Idle resident memory | 400 MB |
| Resident memory, typical session | 1.5 GB |
| Resident memory, peak | 2.5 GB |
| Idle CPU, mean over 60 s | 2%, of which the framework's idle loop is about 1.3% |

The 8 ms figures are one frame at 120Hz: a task that exceeds them drops a frame, which is
what the product's central claim of being measurably snappier rests on. Cold start excludes
indexing, plugin activation and language servers, none of which may block a file being
rendered. The completion budget is stated end to end against a reference round trip of 80 ms,
which leaves 170 ms for the client, serialisation, server work and render combined. It is
stated that way rather than as an absolute figure because the language server is frequently
on another machine, and no architecture beats the propagation delay to it.

The peak memory budget is set so that at least 5 GB remains for the operating system and
other applications on an 8 GB machine; it is the reason the figure is not simply "as much
as is free". Installed disk footprint MUST be reported per release and justified when it
grows, since low-specification machines are frequently also storage-constrained.

Collections rendered in the interface MUST be virtualized or paginated when their size is
unbounded by input. Polling loops, watchers, and animations MUST stop when the window is
hidden or the application is idle, because the idle CPU budget is what determines whether
the machine stays responsive for everything else the user is running.

## Development Workflow and Quality Gates

`plan.md` is the sole source of the technology stack for a feature. Specifications
describe what and why and MUST remain free of stack choices; adapter technology is
chosen at plan time and MUST NOT appear in domain or application code.

Design documents live at `docs/system-designs/<NNN>-<feature-slug>.md`, where `<NNN>` is
the feature number used by the specification for the same feature, so that the design,
the spec, and the plan for one feature are correlatable by name.

Three artifacts govern a feature and MUST NOT duplicate each other:

- `spec.md` states what the feature does and why, and MUST remain free of stack and
  structure choices.
- The design document states how the system is shaped: boundaries, ports, use cases,
  domain model, data flow, and failure behavior.
- `plan.md` states how the work is built and remains the sole source of the technology
  stack. Where the design document and `plan.md` disagree about technology, `plan.md`
  wins and the design document MUST be corrected in the same change.

The design document MUST NOT restate the stack; it references `plan.md` for that.

The high-level design MUST contain: the problem and the context it changes; a diagram of
the feature's place in the system showing components, external systems, and trust
boundaries; the primary scenario traced end to end; the failure modes and what the system
does in each; the expected impact on the resource budgets in Architectural Constraints;
and the alternatives considered with the reason each was rejected.

The low-level design MUST contain: the domain entities and value objects with their
invariants; each use case with its input and output types; each inbound and outbound port
with its signature; each adapter with the technology it wraps, referencing `plan.md`; the
error taxonomy and where errors are translated across boundaries; persistence and
migration impact; the test plan expressed in the tiers defined by Principle V; and the
observability signals the feature emits.

Diagrams MUST be authored as text, in Mermaid or an equivalent diff-readable format.
Rationale: an image cannot be reviewed in a diff, and a diagram that cannot be reviewed
in a diff stops matching the system within a release or two.

A design document for a user-facing feature MUST include a prototype mapping section
naming the region of `mockups/Vulcan-IDE.html` the feature implements, the tokens it
consumes, and every prototype extension the feature introduces with its sign-off status.

`specs/features-map.md` uses these conventions. Features are listed with an immutable
`F<NNN>` identity, the dependencies they declare, and a `[P]` marker when they are
independent of their neighbours. Each feature carries a checkbox, and each subfeature
carries its own checkbox beneath it.

Spec Kit assigns specification directory numbers sequentially as `/speckit-specify` is
run, which is a different numbering from the `F<NNN>` identities and will diverge from
them as features are inserted. When a feature is specified, its map entry MUST record the
resulting `specs/<NNN>-<slug>` path, so that the two numbering schemes stay reconcilable
and a reader can move between the backlog and the specifications without guessing.

The following gates apply to every change:

1. Boundary check: an automated import-boundary rule MUST run in CI and fail the
   build on any inward-dependency violation described in Principle I. Until such a
   rule exists for a given language in this repository, adding it is a prerequisite
   of the first feature written in that language.
2. Port coverage: any new external dependency introduced by a change MUST arrive with
   a port, an adapter, and a contract test suite in the same change.
3. Test gate: domain and use-case tests MUST pass without network, database, or
   filesystem access. A test in those layers that requires infrastructure is a defect
   in the boundary, not in the test.
4. Review gate: a reviewer MUST be able to name, from the diff alone, which layer each
   added file belongs to. Files that do not fit a layer MUST be relocated or the
   design revisited before merge.
5. Performance gate: the budget suite from Architectural Constraints MUST run in CI on a
   runner constrained to the baseline machine. Exceeding any budget fails the build. A
   regression of more than 10 percent on any budget metric, even while still under
   budget, MUST be explained in the pull request before merge.

6. Test completeness gate: every pull request that changes behavior MUST state which
   test tiers it touches and carry the corresponding tests, or name the omitted tier and
   the reason it does not apply. A reviewer MUST treat an unexplained missing tier as a
   blocking finding.

7. Design gate: a feature's design document MUST be committed before its first
   implementation commit, and every implementation pull request MUST link to it. A pull
   request that diverges from the linked document without updating it in the same change
   is a blocking finding.

8. Fidelity gate: user-facing changes MUST pass a visual regression comparison against
   renders of `mockups/Vulcan-IDE.html` at the prototype's viewport, and MUST pass a lint
   rule rejecting design values that are not extracted prototype tokens. An unapproved
   visual diff or an off-token literal fails the build.

9. Sequence gate: before a feature is specified, its declared dependencies in
   `specs/features-map.md` MUST all be checked, and the feature's own entry MUST be
   unchecked. A specification produced out of sequence, or produced for a feature whose
   dependencies are incomplete, is a blocking finding. This gate is enforced automatically
   by the `featuremap` extension's mandatory `before_specify` hook, which refuses to create
   a specification directory when the sequence is violated.

Complexity that violates a principle MUST be justified in the feature's plan under an
explicit exception entry naming the principle, the reason, and the simpler alternative
that was rejected. An unjustified violation is a blocking review finding.

## Governance

This constitution supersedes conflicting practices, conventions, and habits in this
repository. Where a tool default and this document disagree, this document wins and
the tool is reconfigured.

Amendments MUST be made by editing this file, MUST state the rationale in the sync
impact report at the top, and MUST include a migration note when existing code is put
out of compliance.

Versioning follows semantic versioning of governance intent:

- MAJOR: a principle is removed, or redefined in a way that invalidates compliant code.
- MINOR: a principle or section is added, or existing guidance is materially expanded.
- PATCH: clarification, wording, or typo correction that does not change what is required.

Compliance is reviewed at three points: at `/speckit-plan`, where the plan records how
the feature satisfies each principle; at review time, through the gates above; and on
amendment, when open work is assessed against the new version.

**Version**: 4.1.0 | **Ratified**: 2026-09-18 | **Last Amended**: 2026-09-19
