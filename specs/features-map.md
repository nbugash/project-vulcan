# Vulcan Feature Map

Source of truth: `vulcan.md`. Work this file from top to bottom. Every item's
prerequisites sit above it, so the reading order is the build order.

Run `/speckit-specify` once per top-level feature, in the order listed. Spec Kit assigns
`specs/NNN-*` numbers sequentially as you run them, so working downward keeps the spec
numbers aligned with the F-numbers here.

## How to use this checklist

- A subfeature box is checked when that slice is merged and its tests pass.
- A feature box is checked only when every subfeature is checked AND the feature has
  cleared the constitution's gates: a spec, a design document under `docs/system-designs/`,
  tests at every applicable tier, and gates 1 through 8 green. Partial work stays unchecked.
- `[P]` marks features with no dependency on each other. They can run in parallel once the
  feature they both depend on is done. Everything unmarked is strictly sequential.
- Standing constraints that apply to every feature are at the bottom of this file. Read
  them before the first spec.
- Each feature carries a `Spec:` line recording the `specs/<NNN>-<slug>` directory created
  for it. Spec Kit numbers those directories sequentially as specs are written, which is a
  different sequence from the `F<NNN>` identities here; the recorded path is what keeps the
  two reconcilable. It is written automatically by
  `python3 .specify/extensions/featuremap/scripts/python/feature_map.py record-spec F001 specs/002-slug`.
- `/speckit-specify` runs the sequence gate before doing anything, through the
  `before_specify` hook registered in `.specify/extensions.yml`. Invoked with no argument it
  selects the first unchecked feature whose dependencies are all checked; invoked with an
  identity it refuses to proceed when that feature's dependencies are incomplete.

## Tier 0 - Engineering baseline

The constitution references gate tooling that does not exist yet. Building it first stops it
from being improvised under pressure during feature work.

This tier also builds the shell. That is deliberate and it is not a fixture: the budgets in
Principle VI are the product's central claim, and measuring keystroke-to-paint or idle memory
against a placeholder proves nothing. A real shell makes the first measurement honest, and it
tests the rendering framework before twenty features depend on it.

Scope of the shell here is presentation and local interaction. Every region the prototype
composes is rendered and responds: panels resize and collapse, tabs switch, the palette opens
and filters, dialogs open, the density and layout props take effect. Nothing behind them is
real: no files are opened, no language server is contacted, no command executes. Wiring each
control to behaviour belongs to the feature that owns that behaviour.

Because the shell is user-visible, the end-to-end tier is not waived for this feature. Because
it is built before the plugin host exists, it is written outside the plugin model and re-hosted
by F002.

- [x] **F000 engineering-baseline**
  - Spec: specs/001-engineering-baseline
  - [x] Import-boundary lint rule, failing the build on inward-dependency violations (gate 1)
  - [x] Runner constrained to 6 cores and 8 GB, with limits applied rather than requested.
        The Linux runner writes `cpu.max` and `memory.max` into a cgroup delegated to it,
        reads both back, counts the cores it was actually given, and refuses to judge when
        any of that disagrees with the baseline. macOS has no cgroups, so the Apple Silicon
        runner still measures an 11-core, 18 GB M3 Pro: a machine larger than the baseline,
        which makes its numbers optimistic rather than wrong
  - [x] Budget metric collection: cold and warm start, resident memory, longest UI-thread
        task, input to first paint. Cold start evicts the product's own binary from the page
        cache with `posix_fadvise` before launching, so it measures a read from disk rather
        than a read from memory; warm start is the relaunch that follows. Idle processor was
        removed in constitution v4.2.0 as premature (gate 5)
  - [x] Network profiles injecting 0, 10, 30 and 80ms round trip for latency verification
  - [x] Recover and vendor the prototype's font binaries, which are currently absent from `mockups/`
  - [x] Design-token extraction script reading the prototype under `mockups/`
  - [x] Visual regression baseline and diff reporting against the prototype (gate 8)
  - [x] Off-token design value lint rule
  - [x] Every region of the prototype rendered, matching it exactly at its viewport
  - [x] Every design value in the shell sourced from extracted tokens, none invented
  - [x] Prototype extensions recorded and submitted for sign-off where the mock is silent
  - [x] Sequence gate automation: `featuremap` extension `before_specify` hook (gate 9)
  - [x] Shell chrome at the prototype's fixed heights: toolbar, remote banner, rail, tab strip, breadcrumbs, status bar
  - [x] Tool windows and dock, with their tab strips, resizable and collapsible
  - [x] Editor surface presentation: tabs, gutter, inlay hints, completion popup
  - [x] Command palette that opens, filters and closes
  - [x] Run configuration and language pack dialogs
  - [x] Density, tool side and performance readout props switching at runtime

## Tier 1 - Walking skeleton

Together these answer the product's riskiest question: whether a 6-core, 8 GB client
driving a remote language server stays inside the Principle VI budgets.

The product is three layers, not two. Language toolchains are other people's programs and
run remote. The workspace engine daemon is ours and runs remote: it supervises those
toolchains, owns the index, and multiplexes one connection to the client. The client is ours
and runs local: buffer, undo, highlighting and rendering. The client is dumb about semantics
and smart about text, because no amount of remote hardware moves the speed of light and the
keystroke path cannot pay a round trip. If the answer is
no, it must be discovered here and not after thirty features assume otherwise.

- [ ] **F001 plugin-host-runtime** (depends: F000)
  - Spec: not yet specified
  - [ ] Plugin manifest format and schema validation
  - [ ] Extension point registry and contribution model
  - [ ] Dependency resolution and version compatibility checks
  - [ ] Lazy activation triggers
  - [ ] Lifecycle: load, activate, deactivate, unload
  - [ ] Failure isolation, so one faulty plugin cannot bring down the host
  - [ ] Host API surface versioning policy
- [ ] **F002 app-shell-workspace** (depends: F001)
  - Spec: not yet specified
  - [ ] Shell re-hosted through the plugin host's extension points
  - [ ] Layout persistence across sessions
  - [ ] Command registry and dispatch, with controls wired to real commands
  - [ ] Keymap routing with an IntelliJ-compatible default
  - [ ] Prototype extensions for states absent from the mock, submitted for sign-off
- [ ] **F003 remote-workspace-session** (depends: F001)
  - Spec: not yet specified
  - [ ] SSH connection lifecycle and authentication, with no secrets at rest in plaintext
  - [ ] Remote filesystem port: list, read, write, stat
  - [ ] Opening a project against a remote host
  - [ ] File watching and change notification at repository scale
  - [ ] Reconnect with backoff, and defined behaviour while disconnected
  - [ ] Channel multiplexing over a single connection
- [ ] **F004 editor-core** (depends: F002)
  - Spec: not yet specified
  - [ ] Rope or gap buffer with an allocation-free edit path
  - [ ] Keystroke to paint within 8ms p99, never waiting on the network
  - [ ] Undo and redo grouped by typing run rather than by keystroke
  - [ ] Virtualised viewport rendering for large files
  - [ ] Scroll tick to paint measured against its 8ms p99 budget. Carried from F000, which
        built the gate but has nothing that scrolls
  - [ ] Caret and selection model
  - [ ] Find and replace within a file
  - [ ] Encoding and line-ending handling
- [ ] **F005 syntax-highlighting** (depends: F004)
  - Spec: not yet specified
  - [ ] Grammar API exposed as an extension point
  - [ ] Incremental re-highlight on edit
  - [ ] Token to theme mapping through prototype tokens
  - [ ] One default grammar shipped as a plugin
  - [ ] Degradation strategy for very large files
- [ ] **F037 workspace-engine-daemon** [P] (depends: F003)
  - Spec: not yet specified
  - [ ] Daemon bootstrap over the remote session, with version negotiation against the client
  - [ ] Language server supervision in-process: spawn, restart, crash recovery, lifecycle
  - [ ] Index and symbol cache owned remotely, surviving client disconnection and reconnection
  - [ ] Multiplexed protocol over one connection, with batching, coalescing and cancellation
  - [ ] Authentication and least-privilege execution, reachable only through the tunnel
  - [ ] Round-trip count per operation measured and reported at the 0, 10, 30 and 80ms profiles
- [ ] **F006 lsp-client-core** (depends: F005, F037)
  - Spec: not yet specified
  - [ ] Protocol client speaking to the workspace engine daemon, not to servers directly
  - [ ] Request cancellation and coalescing, so round-trip count is a measured property
  - [ ] Capability negotiation
  - [ ] Incremental document synchronisation
  - [ ] Completion, hover, diagnostics, go to definition
  - [ ] Latency budgets verified at 0, 10, 30 and 80ms round trip
  - [ ] Go via gopls as the lead language

## Tier 2 - Usable editor

- [ ] **F007 code-navigation** (depends: F006)
  - Spec: not yet specified
  - [ ] Go to definition, implementation, and type definition
  - [ ] Find usages across the project
  - [ ] Navigation history stack
  - [ ] Peek view without leaving the current file
- [ ] **F008 global-search** [P] (depends: F006, F037)
  - Spec: not yet specified
  - [ ] Fuzzy lookup of files, symbols, actions and settings
  - [ ] Full-text search executed by the daemon against its own index
  - [ ] Result ranking and incremental display
  - [ ] Cancellation of in-flight queries
  - [ ] Fuzzy file open measured against its 50ms budget across 100,000 files, and project
        text search against its 500ms first-results budget. Carried from F000
- [ ] **F009 completion-ux** [P] (depends: F006)
  - Spec: not yet specified
  - [ ] Ranking and filtering
  - [ ] Signature help
  - [ ] Documentation popups
  - [ ] Snippet insertion and commit characters
  - [ ] Asynchronous rendering that never blocks typing
  - [ ] Completion popup measured against its 250ms p95 budget at a round trip of 80ms or
        less. Carried from F000, which has no language server to ask
- [ ] **F010 diagnostics-and-quickfix** [P] (depends: F006)
  - Spec: not yet specified
  - [ ] Severity model
  - [ ] Gutter and inline presentation
  - [ ] Problems panel
  - [ ] Quick fixes and suppression
  - [ ] Debounce tuned against the 300ms to 2s diagnostics budget
  - [ ] Diagnostics after a typing pause measured against that budget. Carried from F000
- [ ] **F011 settings-keymaps-themes** [P] (depends: F002)
  - Spec: not yet specified
  - [ ] Settings schema, storage and scoping across application and project
  - [ ] Keymap model with an IntelliJ-compatible default
  - [ ] Theme plugin contract consuming prototype tokens
  - [ ] Import and export of settings
- [ ] **F012 integrated-terminal** [P] (depends: F003)
  - Spec: not yet specified
  - [ ] PTY attached to the remote host
  - [ ] Multiple concurrent sessions
  - [ ] Resize and reflow
  - [ ] Send selection to terminal
  - [ ] Copy, paste and shell integration
- [ ] **F013 vcs-git-core** (depends: F003)
  - Spec: not yet specified
  - [ ] Repository status against a remote working copy
  - [ ] Stage, unstage, commit
  - [ ] Branch management
  - [ ] Fetch, pull, push
  - [ ] History log and commit tree visualisation
- [ ] **F014 vcs-diff-merge** (depends: F013)
  - Spec: not yet specified
  - [ ] Side-by-side and inline diff
  - [ ] Stage and revert by hunk
  - [ ] Three-way merge view
  - [ ] Conflict resolution flow

## Tier 3 - Integrated development

- [ ] **F015 debug-client-core** (depends: F006)
  - Spec: not yet specified
  - [ ] Debug adapter client and transport over the remote session
  - [ ] Breakpoint model: line, conditional, logpoint
  - [ ] Stepping and execution control
  - [ ] Call stack navigation
  - [ ] Variable inspection and setting values
  - [ ] Watches and live expression evaluation
  - [ ] Remote attach and launch, with Delve as the first adapter
- [ ] **F016 run-configurations** (depends: F001, F015)
  - Spec: not yet specified
  - [ ] Configuration model and storage
  - [ ] Extension point for plugin-defined configuration types and dialogs
  - [ ] Run and debug dialog
  - [ ] Sharing configurations through the repository
- [ ] **F017 build-integration** [P] (depends: F016)
  - Spec: not yet specified
  - [ ] Build tool port and task discovery
  - [ ] Build console with cancellation
  - [ ] Error parsing back into editor markers
  - [ ] Incremental build support
- [ ] **F018 test-runner** [P] (depends: F016)
  - Spec: not yet specified
  - [ ] Test discovery
  - [ ] Execution and live progress
  - [ ] Visual test tree
  - [ ] Failure to source navigation
  - [ ] Coverage reporting
  - [ ] Rerun failed only
- [ ] **F019 refactoring-core** (depends: F007)
  - Spec: not yet specified
  - [ ] Rename across the project
  - [ ] Extract method
  - [ ] Move class or module
  - [ ] Preview with conflict detection
  - [ ] Single-step undo for a whole refactoring
  - [ ] Correct behaviour against a remote codebase

## Tier 4 - Language packs

One spec each, all instantiating the same template: grammar, language server
configuration, debug adapter, build tool hooks, and environment management where the
language needs it. Each pack is a plugin, which is what proves the plugin API is
sufficient for a third party. All are parallel once F015 and F017 are done.

- [ ] **F020 language-pack-jvm** [P]
  - Spec: not yet specified
  - [ ] jdtls configuration and remote lifecycle
  - [ ] JPDA and JDWP debugging
  - [ ] Gradle and Maven integration
  - [ ] Pack conformance tests
- [ ] **F021 language-pack-python** [P]
  - Spec: not yet specified
  - [ ] Language server configuration
  - [ ] debugpy integration
  - [ ] Virtual environment switching for venv, conda and poetry from the status bar
  - [ ] Pack conformance tests
- [ ] **F022 language-pack-rust** [P]
  - Spec: not yet specified
  - [ ] rust-analyzer configuration
  - [ ] LLDB or CodeLLDB debugging
  - [ ] Cargo integration
  - [ ] Pack conformance tests
- [ ] **F023 language-pack-c-cpp** [P]
  - Spec: not yet specified
  - [ ] clangd configuration
  - [ ] gdbserver or lldbserver debugging
  - [ ] CMake integration
  - [ ] Pack conformance tests
- [ ] **F024 language-pack-elixir** [P]
  - Spec: not yet specified
  - [ ] ElixirLS configuration
  - [ ] BEAM debugging
  - [ ] Mix integration
  - [ ] Pack conformance tests
- [ ] **F025 language-pack-zig** [P]
  - Spec: not yet specified
  - [ ] ZLS configuration
  - [ ] LLDB debugging
  - [ ] Zig build integration
  - [ ] Pack conformance tests
- [ ] **F026 language-pack-javascript** [P]
  - Spec: not yet specified
  - [ ] Language server configuration
  - [ ] V8 Inspector Protocol over a tunnelled connection
  - [ ] Package script integration
  - [ ] Pack conformance tests

Go ships inside F006 and is retrofitted into this template when the second pack lands, so
the template is derived from two real cases rather than generalised from one.

## Tier 5 - Paradigm-specific depth

Each depends on its language pack and on F015. All parallel.

- [ ] **F027 reactive-stream-debugging** [P]
  - Spec: not yet specified
  - [ ] Reactor, RxJava and Kotlin Flow pipeline detection during a debug session
  - [ ] Marble or timeline presentation of emissions
  - [ ] Subscription and backpressure state inspection
- [ ] **F028 concurrency-visualizer** [P]
  - Spec: not yet specified
  - [ ] Goroutine monitor
  - [ ] BEAM process monitor
  - [ ] Presentation of thousands of routines without breaching client budgets
  - [ ] Filtering and grouping
- [ ] **F029 hot-code-reload** [P]
  - Spec: not yet specified
  - [ ] Code injection into a running Elixir application
  - [ ] Reload failure reporting and recovery
- [ ] **F030 macro-expansion** [P]
  - Spec: not yet specified
  - [ ] Inline expanded view for Rust macros
  - [ ] Inline expanded view for C and C++ macros
- [ ] **F031 bytecode-decompiler** [P]
  - Spec: not yet specified
  - [ ] Decompiled source when navigating into a compiled JVM dependency
  - [ ] Source attachment when available, decompilation as fallback
- [ ] **F032 interactive-repl** [P]
  - Spec: not yet specified
  - [ ] REPL session attached to the remote host
  - [ ] Execute selection from the editor
  - [ ] Python and Elixir as the first two

## Tier 6 - Product platform

- [ ] **F033 plugin-distribution** (depends: F001)
  - Spec: not yet specified
  - [ ] Plugin registry and discovery
  - [ ] Install, update and removal
  - [ ] Sideloading
  - [ ] Signature verification and trust model
  - [ ] Compatibility gating against the host API version
- [ ] **F034 claude-integration** [P] (depends: F002, F006)
  - Spec: not yet specified
  - [ ] Assistant surface inside the IDE
  - [ ] Explicit context boundary and consent model
  - [ ] Secret redaction before any request leaves the machine
  - [ ] Behaviour when the service is unreachable
  - [ ] Usage visibility for the developer
- [ ] **F035 observability-and-crash-reporting** [P] (depends: F002)
  - Spec: not yet specified
  - [ ] Structured diagnostics and log export
  - [ ] Opt-in crash reporting
  - [ ] Field measurement of the Principle VI budgets on real hardware
  - [ ] Privacy controls and data retention
- [ ] **F036 packaging-and-update** (depends: F035)
  - Spec: not yet specified
  - [ ] Linux packaging
  - [ ] macOS packaging with signing and notarisation
  - [ ] Delta automatic update
  - [ ] Rollback to the previous version

## Adding a feature

Identities are never renumbered and never reused. A new feature takes the next unused
number and is placed where it belongs in build order; the number is its name, the position
is its sequence.

Use the command rather than editing by hand, because the parser that drives the sequence
gate is strict about format and the script refuses insertions that would break the map:

```
/speckit-specify ADD a workspace-wide symbol index that keeps navigation fast on large
remote repositories
```

That resolves to `/speckit-featuremap-add`, which proposes a slug, a position, the
dependencies, the `[P]` marker and three to six subfeatures, asks only what it cannot
infer, previews the block, and writes it after confirmation. Directly, the same thing is:

```bash
python3 .specify/extensions/featuremap/scripts/python/feature_map.py add \
  --slug workspace-indexing \
  --after F006 \
  --depends F003,F006 \
  --parallel \
  --subfeature "Incremental index build on the remote host" \
  --subfeature "Index invalidation on external file change" \
  --dry-run
```

Drop `--dry-run` to apply. The script assigns the identity, refuses a dependency
positioned below the insertion point, re-verifies the file afterwards, and rolls the write
back if integrity would break.

Three consequences worth knowing before you insert:

- **Inserting high in the file reprioritises immediately.** If everything above the new
  entry is checked, `/speckit-specify` with no argument resolves to the new feature next.
  Position is a claim about build order, not a wish list.
- **Adding a subfeature to an already-checked feature unchecks it.** `verify` fails on a
  checked parent with unchecked children, so the feature returns to the queue. That is
  deliberate: under Principle IX a checked box means done, and it no longer is. It also
  triggers the enhancement rules in Principles VII and V, so update that feature's existing
  design document rather than writing a second one, and add characterisation tests before
  modifying code that has none.
- **Nothing infers reverse dependencies.** If existing features should now depend on the
  new one, edit their `(depends: ...)` lines yourself. `verify` detects a wrong dependency,
  never a missing one.

Removing a feature follows the same rule from the other direction: strike it with a
recorded reason rather than deleting it, so its absence reads as a decision.

## Standing constraints

These apply to every feature above and are governed by
`.specify/memory/constitution.md` v2.2.0.

### Latency

`vulcan.md` asks for "< 10ms of latency" on remote development. That cannot hold as a
round trip: fibre carries signal at roughly 204 km per millisecond, so a host 500 km away
costs about 7ms of propagation alone, before switching, last mile and server compute.
Keystroke to photons under 10ms is unreachable on commodity hardware regardless, since USB
polling costs 1 to 8ms and display scan-out and pixel response cost 10 to 20ms before the
application is involved. The budgets below separate what the application controls from
what the network and the display impose. The constitution is their only home now that the
architecture document they were taken from has been deleted.

| Path | Budget | Basis |
| --- | --- | --- |
| Keystroke to paint | 8ms p99 | One frame at 120Hz |
| Scroll tick to paint | 8ms p99 | One frame at 120Hz |
| Longest task on the UI thread | 8ms | Follows from the frame budget |
| Highlight update after an edit | 16ms, off the UI thread | Constitution Principle VI |
| Cold start to a rendered file | 300ms | Excludes indexing, plugins and language servers |
| Fuzzy file open across 100,000 files | 50ms per query | Constitution Principle VI |
| Project text search, first results | 500ms | Constitution Principle VI |
| Completion, go to definition, hover, symbol search | 250ms p95 end to end, at a round trip of 80ms or less | Product decision: the user-facing near-realtime promise |
| Implied overhead above round trip for those operations | 170ms for client, serialisation, server compute and render combined | Derived from the row above |
| Diagnostics after a typing pause | 300ms to 2s, never blocking | Constitution Principle VI |
| Idle resident memory | 400MB | Constitution Principle VI |
| Resident memory, typical / peak | 1.5GB / 2.5GB | Leaves at least 5GB for the rest of the machine |

The client MUST be the authority on buffer contents; echo never waits on the network. The
250ms figure applies to discrete request-response operations only. At 60 words per minute a
typist produces a keystroke every 200ms, so a 250ms echo would render each character after
the next had been typed. Continuous paths stay on the 8ms frame budget.

The baseline machine is 6 CPU cores and 8 GB of memory, of A18 Pro class, with a 120Hz
display. Its cores are asymmetric, two performance and four efficiency, so work scheduled as
though six equal cores were available will not meet these budgets.

### Prototype coverage

The prototype under `mockups/` composes one viewport, one theme and fixed panel widths, with no
media queries and no theme variants. An IDE has dockable, resizable panels. Most window
states this product needs are therefore prototype extensions under Principle VIII and
require sign-off as they are designed. F002 establishes that path; every later feature with
a visual surface uses it.

### Out of scope

Windows support (confirmed out of scope), cloud-hosted workspaces beyond a user-supplied remote host, collaborative
or multi-user editing, a web client, and assistant features beyond F034. None appear in
`vulcan.md`; they are listed so their absence is a decision rather than an oversight.
