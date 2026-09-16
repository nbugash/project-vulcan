# Vulcan Constitution

Vulcan is a desktop IDE with IntelliJ's look, feel, and workflow richness, built to be
measurably snappier than every current alternative when editing any language, locally or
against a remote machine, on a 5-CPU / 12 GB laptop running macOS, Linux, or Windows, with a
plugin system architected from day one.

This constitution governs how that claim is kept. Its evidence base is `vulcan-system-design.md`;
each principle cites the sections it derives from.

Where this document and the design doc disagree, the disagreement is a defect in one of them and
MUST be resolved explicitly rather than by silent deference. On **obligations** — what is
required, forbidden, budgeted, or gated — this document governs. On **facts** — measurements,
arithmetic, and the documented behaviour of reference implementations — the design doc governs,
and a principle that contradicts an established fact MUST be corrected here rather than
enforced. On **interface** — appearance, layout, and interaction — the approved mock at
`mockups/Vulcan IDE.html` governs, within the limits Principle IX sets.

## Core Principles

### I. The Keystroke Path Is Sacred (NON-NEGOTIABLE)

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

### II. Snapshots, Cancellation, Versioned Results

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

### III. No Feature Waits on a Server or an Index

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

### IV. Test-First (NON-NEGOTIABLE)

Behaviour is specified as a failing test before it is implemented: red, then green, then refactor,
with the red observed rather than assumed.

- Every feature MUST open with the three tests its specification columns imply (see *Development
  Workflow and Quality Gates*): a latency assertion against its budget, a cold-index test proving
  the feature works with nothing indexed, and a remote-mode test proving it works across the agent
  boundary. A feature whose three columns are filled but whose three tests are missing has not
  been started.
- The test MUST be run and seen to fail, for the intended reason, before implementation begins. A
  test that has never failed has demonstrated nothing.
- Every bug fix MUST open with a regression test that reproduces the defect against the unfixed
  code.
- The failure modes in `vulcan-system-design.md` §18 are a standing automated matrix, not a manual
  checklist: server crash, server hang, oversized file, remote disconnect mid-edit, agent crash,
  agent version mismatch, misbehaving extension, and IDE crash recovery. Each MUST assert both
  that no typed text is lost and that the UI stays responsive.
- The Principle II invariants MUST be asserted mechanically rather than by inspection: that
  background work receives snapshots and never live references, that every result carries the
  version it was computed against, that stale results are discarded or anchor-mapped, and that
  cancellation actually fires on edit.
- Tests MUST pass on macOS, Linux, and Windows. Passing only on the development platform is not
  passing.

**Scope.** Test-first governs all logic: buffer and anchors, the display map, keymap resolution
and command dispatch, the VFS and indexer, LSP and DAP client behaviour, the remote channel codec,
and extension host policy. It does not govern the rendering leaf (glyph rasterization, GPU
submission) or OS input plumbing (IME composition, window management), where correctness is
established instead by golden-image comparison, the latency harness of Principle V, and named
manual platform checks. These exemptions MUST be narrow and enumerated in the plan. "Hard to test"
is not an exemption; "not expressible as an assertion" is, and it is rare.

**Rationale:** Every guarantee this constitution makes is invisible when it holds and reproducible
only under timing that is impractical to hit by hand — a stale result landing three keystrokes
late, a cancellation that silently did not fire, an anchor drifting after an insertion above, a
disconnect during an unsaved edit. Clicking around cannot find these; they are indistinguishable
from correct behaviour until a user hits one. Writing the test first is what forces the invariant
to be stated in a form a machine can check, and Principle V already establishes that this project
gates on machine-checked properties rather than on review. (§10, §18, §19.2)

### V. Budgets Are CI Gates, Not Aspirations

Performance that is "felt" drifts a millisecond at a time. Vulcan measures instead.

- Keystroke-to-paint and frame timing MUST be instrumented from the first spike, using
  Typometer-style measurement.
- The budgets in `vulcan-system-design.md` §16 are binding. A p99 regression against them MUST
  fail CI. It is not a review discussion.
- Latency MUST be reported at p99 and p999, never as a mean alone. Jitter is as damaging as
  the mean, and a 3 ms mean with 80 ms spikes feels worse than a steady 8 ms.
- Remote-mode budgets MUST be verified under simulated latency and loss (`tc netem` or Network
  Link Conditioner) at a representative RTT, not on a localhost agent alone.
- Per-server memory and CPU MUST be observable by the user at runtime, not only in tests.

**Rationale:** §16's table is written as design constraints. Without an automated gate the
budgets become documentation, and the product's one differentiating claim erodes invisibly.
(§5.6, §5.8, §16, §19.2)

### VI. Local and Remote Are One Implementation

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

### VII. Bounded Footprint on Modest Hardware

The target machine is 5 CPUs and 12 GB, and the IDE does not get 12 GB. After the OS, a browser,
and container tooling, everything the IDE owns fits in roughly 4–5 GB — most of which belongs to
language servers and build daemons, not to Vulcan.

- The Vulcan process MUST stay within roughly 500 MB idle and roughly 1 GB with a medium project
  open. In remote mode the client MUST stay at or under 500 MB.
- Lazy by default: language servers start on the first file of their language, never on project
  open, and stop after an idle timeout. Grammars, themes, and extensions load on first use.
- Every cache MUST have an explicit ceiling and an eviction policy — glyph atlas, shaped-text
  cache, buffer snapshots, indexes. "Grow until OOM" is prohibited.
- Trigram and symbol indexes MUST be memory-mapped files on disk, so they cost page cache rather
  than heap and survive restart.
- Anything unbounded MUST stream: search results, references, diagnostics, file trees, workspace
  symbols. A 50k-item result MUST NOT be materialized to display 40 rows.
- The background pool MUST be small and low priority (`max(2, cores − 2)`, sized from efficiency
  cores on heterogeneous parts, at background/utility QoS). Indexing MUST throttle on battery and
  under load. Indexing that takes 40 s instead of 20 s is invisible; a stuttering cursor is not.
- Nothing polls and nothing animates continuously. An idle Vulcan MUST draw no frames.

**Rationale:** Footprint matters less for its own sake than because a bloated process means cache
misses and eventually paging, which surfaces as tail latency — a Principle I violation by another
route. This budget is also the decisive argument for the native core in §6.3. (§17.1, §17.2,
§14.8)

### VIII. Extensions Are Data First, Sandboxed Code Second

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
guarantee anything about latency or memory. On a 12 GB machine, isolation is the correct trade.
Dogfooding the API before publishing it is the only way to avoid guessing at its shape.
(§9.11, §17.5)

### IX. The Approved Mock Governs the Interface

`mockups/Vulcan IDE.html` has been approved by the UI designer and by stakeholders. It is the
source of truth for Vulcan's interface. UI work resolves against it, not against individual
judgement.

- Every UI and UX decision MUST be resolved by consulting the mock first. Where it shows an
  answer — layout, spacing, typography, colour, iconography, information architecture,
  interaction affordance, or wording — that answer is the specification. An implementation that
  departs from it is a defect, whether or not the departure looks better.
- The mock's authority covers **appearance and interaction only**. It does not govern
  architecture, performance, or whether a feature exists; those remain with Principles I–VIII and
  `vulcan-system-design.md`. Where a mocked interaction could only be built by violating
  Principle I, **Principle I wins** and the interaction goes back to the designer rather than
  being implemented as drawn.
- Where the mock is **silent** — and it will be, since it depicts ten regions of a much larger
  product — the interface MUST be extended by analogy from what the mock does establish: the
  Nocturne token set, its spacing and type scale, and its existing component patterns. Inventing
  a second visual language is prohibited, as is guessing at a pattern the mock already answers
  somewhere else.
- The mock contains material that is **not product**, and it MUST NOT be implemented as
  features: placeholder content (the `payments-platform` workspace, `OrderService.java`, the
  sample diagnostics and terminal output) and mockup scaffolding (the "Spec pins" toggle, which
  exists to annotate the mock with § references). Where it is unclear whether an element is
  specification or scaffolding, that is a question for the designer, not a developer's call.
- Changing the mock is a **design decision, not an implementation decision**. Because it carries
  designer and stakeholder approval, a developer MUST NOT edit it to match what was built. A
  needed change goes back through the designer and the mock is re-approved.
- Practical note: the file is a bundle, not editable HTML. The application lives in the
  `__bundler/template` script element and must be unpacked before it can be read; in packed form
  it cannot be meaningfully diffed or hand-edited.

**Rationale:** An interface assembled from many individually reasonable local decisions does not
converge on a coherent product — it converges on drift. The mock exists so those decisions were
made once, together, by the people accountable for them; treating it as advisory would hand every
one of them back to whoever happens to be implementing that screen. The narrow scoping matters as
much as the authority: a mock cannot know what a latency budget costs, so it governs what the
interface looks like and how it responds, never what the machine underneath is permitted to do.


## Technology and Platform Constraints

**Architecture (binding).** Vulcan is a single process, GPU-rendered with damage tracking, with
an in-process text model and cheap snapshots. Semantics and debugging are out-of-process behind
LSP and DAP; there is no bespoke per-language semantic engine. These are not stack choices —
they follow directly from Principles I, II, and III, and changing any of them requires amending
those principles.

**Stack constraints (binding).** Whatever implementation stack is selected MUST satisfy all of:

- No garbage collector or runtime pause on the keystroke path, and no pause class that can
  exceed the Principle I budget.
- A deterministic process footprint that meets Principle VII on the target hardware.
- GPU rendering with damage tracking and a glyph atlas, on Metal, DirectX, and Vulkan.
- First-class support on macOS, Linux, and Windows — not a primary platform plus two ports.
- An incremental, error-tolerant syntax layer that re-parses only the edited region, is driven by
  per-language data rather than host code, and can supply highlighting, folding, indentation, and
  in-file symbols with no built index and no language server running.
- The ability to spawn and supervise LSP and DAP server processes on all three platforms.
- A compact binary wire format for the remote channel that decodes off the UI thread.

**The stack selected against these constraints is recorded in the implementation plan and its
architecture decision records, not here.** It MAY change without amending this constitution,
provided every constraint above still holds and the §19.3 thresholds in `vulcan-system-design.md`
are respected. The current selection — Rust, Floem/wgpu, a summary-node rope, tree-sitter,
protobuf-over-SSH, `wasmtime` — lives in `docs/adr/0001-technology-stack.md`.

**Rejected, and not to be re-proposed without a Governance amendment:** any RPC boundary between
keystroke and glyph; Electron or Tauri; C or C++ pursued for a presumed latency win over Rust
(no measurable advantage exists for this workload); Zig (no mature GUI toolkit, no reference
implementation); a JVM core (ruled out on the §17.1 footprint budget — not on GC pauses, which
Generational ZGC has settled).

**Cross-platform (macOS, Linux, Windows):**

- All three platforms MUST be in CI from the first commit. Cross-platform is where hobby IDEs
  die, and it is decided on day one or not at all.
- `#[cfg(target_os)]` MUST NOT appear outside the `platform` module. Everything above that module
  is platform-agnostic.
- Text input and IME (CJK composition, dead keys, macOS press-and-hold) MUST be designed in from
  the start. Retrofitting it is a known late-stage rewrite.
- HiDPI and fractional scaling MUST be tested explicitly at 125% and 150%.

**Non-goals for v1:** a public plugin marketplace; a bespoke per-language semantic engine
(IntelliJ's PSI); AI features; and any feature that adds a step to the keystroke path.

**Acknowledged ceiling:** an LSP-based IDE is as deep as its servers. Vulcan will not match
IntelliJ's deepest cross-file, type-aware refactorings, and the product framing MUST be honest
about it: *IntelliJ-rich workflow, Zed-fast editor, LSP-deep semantics.* This ceiling is
structural and accepted deliberately.

## Development Workflow and Quality Gates

**Every feature specification MUST carry three columns** before it is accepted:

1. **Latency budget** — the §16 target this feature is held to.
2. **Works while indexing** — what the feature does before the index is built (Principle III).
3. **Works remote** — how many network round trips it costs, and what it shows before they
   return (Principle VI).

A feature that cannot fill all three columns is not specified yet, and a feature whose three
columns have no corresponding failing tests has not been started (Principle IV). The columns
and the tests are the same statement written twice — once for the reader, once for CI.

**Stage gates.** Work proceeds through the §19.2 stages, and a stage is not complete until its
gate is measured and passing:

| Stage | Gate | Source |
|---|---|---|
| 0 — spike | p99 keystroke-to-paint under ~10 ms, ideally under one 120 Hz frame (8.3 ms); idle RSS under 200 MB; all three OSes | §19.2 |
| 1 — edit and navigate | Typing latency does not regress while the language server is indexing; per-server memory visible in a resource panel | §19.2 |
| 2 — debug it remotely | Identical keystroke latency local and remote; completion within RTT + 50 ms at a simulated 80 ms RTT with 1% loss; forced disconnect mid-edit loses nothing and does not restart the language server | §19.2 |
| 3 — commit from it | Typing holds its budget while the background diff recomputes on a 50 000-line file; commit, branch, and merge-conflict resolution complete without leaving the IDE, local and remote; local history restores a prior state of a file with git uninvolved | §9.9, §17.4 |
| 4 — plugins | Vulcan's own Java pack, theme, and keymap run through the public extension system with no host-code special case; a deliberately hung extension is killed and reported without the UI missing a frame; per-extension memory and CPU appear in the resource panel | §17.5 |

§19.2 states gates only for Stages 0–2. The Stage 3 and Stage 4 gates above are derived from the
obligations those stages inherit — §9.9's off-thread diff, §17.4's local history, and §17.5's
extension host invariants — rather than invented. Each is adversarial by design: it names the
condition under which the stage's work would fail, not the work itself.

**The acceptance test for the roadmap as a whole is the daily-driver test:** a full day of real
backend work in Vulcan with no fallback to IntelliJ.

**Diagnosing a missed budget.** Per §19.3, a latency miss is presumed to be an architecture defect
until proven otherwise:

- Typing over budget after Principles I and II are applied → a hot-path defect. Fix the hot path.
  Do not consider a rewrite.
- Remote typing slower than local → a network hop has reached the hot path. Find it. Do not tune
  the transport.
- Stage 0 exceeding ~2 months, or stalling on the UI framework → a stack problem, not a principle
  problem. Reopen `docs/adr/0001-technology-stack.md` under its revisit triggers. Only escalate to
  a constitutional amendment if no stack satisfying the Section 2 constraints is reachable.

**Build ergonomics.** Slow compilation on a 5-core machine is an accepted, budgeted cost of a
native core, not a reason to revisit one. Whatever the stack, fast incremental builds are
expected practice rather than an optimization; the specific measures are recorded in
`docs/adr/0001-technology-stack.md`.

## Governance

This constitution supersedes all other development practices for Vulcan. Where a plan, spec, task
list, or review comment conflicts with it, this document wins.

**Amendment procedure.** An amendment MUST be a written change to this file that states the
principle affected, the evidence motivating the change, and the migration consequence for work
already specified or built. Amendments that weaken a performance or isolation guarantee MUST cite
measurement, not preference. Where an amendment changes an obligation, `vulcan-system-design.md`
MUST be updated in the same change so the two never disagree. Where the amendment exists because
the design doc was factually right and a principle here was not, the correction lands here and the
design doc is left alone.

**Versioning policy.** Semantic versioning on this document:

- **MAJOR** — a principle is removed, or redefined in a way that permits something it previously
  forbade.
- **MINOR** — a principle or normative section is added, or existing guidance is materially
  expanded.
- **PATCH** — clarification, wording, or typo fixes that change no obligation.

**Compliance review.** Every plan produced by `/speckit-plan` and every task list produced by
`/speckit-tasks` MUST be checked against these principles before implementation begins, and the
check MUST name the principles reviewed. Complexity that cannot be justified against a principle
is removed, not documented. Principle I violations are blocking and are not subject to
"we will optimize it later."

**Relationship to other documents.** `vulcan-system-design.md` is the evidence base and the
reference for how these principles are implemented; it is not itself governance. Architecture
decision records under `docs/adr/` hold the revisable choices this constitution constrains but
does not make — chiefly the implementation stack. An ADR MUST cite the constraints it is
answering, and MAY be superseded by a later ADR without amending this constitution, provided
those constraints still hold; an ADR that cannot satisfy them requires an amendment first.
`mockups/Vulcan IDE.html` is the approved interface specification governed by Principle IX; it is
authoritative within that scope and silent outside it. `CLAUDE.md`, once written, carries the
working conventions that follow from this constitution.

**Version**: 1.1.0 | **Ratified**: 2026-09-16 | **Last Amended**: 2026-09-16
