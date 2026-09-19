# Vulcan feature inventory

Derived from `vulcan-system-design.md` (§9 components, §14 remote, §17.4 richness, §19.2 stages),
`mockups/Vulcan IDE.html` (ten approved regions), and `.specify/memory/constitution.md` v1.1.0.

Purpose: a complete list to prioritize against, one `/speckit-specify` invocation per row (or per
split, where noted). Status column is for you to fill in.

**Stage** = the §19.2 roadmap stage that item belongs to. **Size** = rough spec effort: S fits one
spec comfortably, M is one large spec, L must be split.

---

## Read this first: most of the order is not actually a choice

There is a hard dependency spine. Roughly 70% of the ordering is forced, and pretending otherwise
wastes prioritization effort. Nothing below the line can be built before the things above it:

```
F3 platform layer ─┐
F1 rope + anchors ─┼─→ F11 concurrency ─→ F5 syntax ─→ E1 editing ─→ F6 latency harness
F2 GPU renderer ───┘                                        │
                                                            ↓
                                        F7 commands + keymaps ─→ everything else
```

`F6` looks like tooling but is a **Principle V obligation** — budgets fail CI, so the harness must
exist before there is anything to gate. `F1`+`F2`+`F3`+`F5`+`E1`+`F6` together *are* Stage 0.

The genuinely open decisions are listed at the bottom.

---

## F — Foundation

| ID | Feature | Source | Stage | Deps | Size |
|---|---|---|---|---|---|
| F1 | Text buffer and document model — rope, cheap snapshots, monotonic versions, anchors, undo transactions grouped by typing run | §9.1 | 0 | — | M |
| F2 | GPU rendering pipeline — damage tracking, text shaping cache, glyph atlas, vsync frame scheduling | §9.3 | 0 | F3 | M |
| F3 | Platform layer — windowing, HiDPI/fractional scaling, **IME and text input**, font enumeration and fallback, path normalization, process spawn, PTY/ConPTY | §17.3 | 0 | — | L |
| F5 | Syntax layer — grammar loading, incremental parse on snapshot, highlights / folds / indents / locals / injections / tags | §9.4 | 0 | F1, F11 | M |
| F6 | **Latency instrumentation and CI gate** — keystroke→paint measurement, p99/p999, budget enforcement in CI on three OSes | §16, Principle V | 0 | F2 | S |
| F11 | Concurrency substrate — background pool, snapshot dispatch, cancellation-on-edit, version stamping, anchor-mapping of stale results | §10 | 0 | F1 | M |
| F4 | Display map — soft wrap, folding, inlay hints, tab expansion, buffer↔screen coordinate mapping | §9.2 | 0–1 | F1 | M |
| F7 | Command registry and keymap resolution — layered (default→platform→user→workspace), context-scoped, resolved per key event | §9.2, §9.10 | 1 | F3 | S |
| F8 | Settings — layered TOML/JSON with schema, hot reload, language-specific overrides | §9.10 | 1 | — | S |
| F9 | Theme system — palette plus scope→style mappings; **Nocturne tokens ported from the mock's CSS to theme data** | §9.10, mock | 1 | F5 | S |
| F10 | Persistence — workspace state, cursor/fold/breakpoint positions, **unsaved buffer contents**, crash recovery | §9.12 | 2 | F1 | S |

## E — Editing

| ID | Feature | Source | Stage | Deps | Size |
|---|---|---|---|---|---|
| E1 | Core text editing — insert/delete, selections, multi-cursor, every command operating over the cursor list | §9.2 | 0 | F1, F4 | M |
| E2 | Viewport and scrolling — visible-line layout, scroll-tick budget, marker rail (errors, warnings, viewport thumb) | §9.2, mock | 0–1 | F4, F2 | S |
| E3 | Editor tabs and breadcrumbs — dirty indicators, tab switching, symbol path | mock Editor | 1 | F7 | S |
| E4 | Gutter — line numbers, VCS change marks, breakpoint dots, fold arrows | mock, §9.9 | 1–3 | F4 | S |
| E5 | Large-file mode — size threshold disables syntax and semantics, plain editing stays fast | §18 | 1 | F5 | S |
| E6 | Local history — per-file rope snapshots grouped by typing run, independent of git, restore to prior state | §17.4, mock rail | 3 | F1, F10 | S |

## N — Navigation and search

| ID | Feature | Source | Stage | Deps | Size |
|---|---|---|---|---|---|
| N1 | Project view / file tree — VFS-backed, watcher-driven, ignore-aware, VCS decorations | §9.5, mock | 1 | L7 | M |
| N2 | Fuzzy file open — name index, <50 ms on 100k files | §16, mock | 1 | L7 | S |
| N3 | Search Everywhere palette — tri-mode files / symbols / commands, ⌘K and ⌘⇧P | mock Palette | 1 | N2, F7, N4 | M |
| N4 | Structure outline — in-file symbols from grammar tags, available while indexing | §9.4, mock | 1 | F5 | S |
| N5 | Project text search — streaming results, scanning or trigram-backed | §9.5, §16 | 1 | L7 | M |
| N6 | **Structural search and replace** — grammar query plus replacement, across every language with a grammar | §17.4, mock | **none** | F5 | M |
| N7 | Go to definition, find references, workspace symbols | §9.6, §12 | 1 | L1, L2 | M |

## L — Language intelligence

| ID | Feature | Source | Stage | Deps | Size |
|---|---|---|---|---|---|
| L1 | LSP client lifecycle — on-demand spawn per language, initialize/capabilities, restart with backoff, idle stop | §9.6, §18 | 1 | F11 | M |
| L2 | Document synchronization — didOpen / didChange with versions / didSave / didClose | §9.6 | 1 | L1, F1 | S |
| L3 | Completion — cancellation on keystroke, multi-source merge and ranking, detail resolved separately, stale results labelled | §9.6, mock | 1 | L2 | M |
| L4 | Diagnostics — merged by source, stale rows labelled and never awaited, positions anchor-mapped | §9.6, mock | 1 | L2 | S |
| L5 | Hover, inlay hints, semantic tokens, document symbols | §9.6 | 1 | L2 | M |
| L6 | Code actions, rename, formatting | §9.6 | 1–3 | L2 | M |
| L7 | Workspace, VFS and indexing — snapshots, watcher, ignore rules; filename / symbol / trigram indexes, incremental, memory-mapped, **never blocking a feature** | §9.5 | 1 | F11 | L |

## D — Debug and run

| ID | Feature | Source | Stage | Deps | Size |
|---|---|---|---|---|---|
| D1 | Run configurations — declarative, storable, shareable, convertible to a debug launch | §9.8 | 2 | F8 | S |
| D2 | DAP client — launch/attach, breakpoints stored as anchors, stepping, lifecycle events | §9.7 | 2 | F11, D1 | M |
| D3 | Debug UI — call frames, lazily-fetched variables, watches, hover-to-evaluate | §9.7, mock | 2 | D2 | M |
| D4 | Integrated terminal — PTY, VT emulator, rendered through the same GPU text pipeline | §9.8, mock | 2 | F3, F2 | M |
| D5 | Tasks — run, stream output, surface exit codes | §9.8 | 2 | D1 | S |

## V — Version control

| ID | Feature | Source | Stage | Deps | Size |
|---|---|---|---|---|---|
| V1 | Git status and tree decorations | §9.9 | 3 | N1 | S |
| V2 | Change gutters — background diff against HEAD on the buffer snapshot, debounced | §9.9 | 3 | F11, E4 | S |
| V3 | Diff view — side-by-side HEAD ↔ working tree | §9.9, mock | 3 | V2 | S |
| V4 | Stage, commit, push | §9.9, mock | 3 | V1 | S |
| V5 | Branch operations | §9.9 | 3 | V1 | S |
| V6 | Blame | §9.9 | 3 | V1 | S |
| V7 | Three-way merge editor — **also required by R5's reconnect conflict path** | §9.9, §14.6 | 3 | V3 | M |

## R — Remote

| ID | Feature | Source | Stage | Deps | Size |
|---|---|---|---|---|---|
| R1 | Agent lifecycle — `ssh host vulcan-agent --stdio`, binary pushed on first connect and cached, version negotiation | §14.7 | 2a | F3 | M |
| R2 | Channel protocol — one multiplexed connection, eight logical channels, length-prefixed binary frames, per-request cancellation | §14.4 | 2a→2b | R1 | L |
| R3 | Remote VFS and shadow buffers — versioned copies on the agent, same buffer type as local | §14.3 | 2a | R2, F1 | M |
| R4 | Remote LSP/DAP hosting — Local variants run on the agent, shaped results cross the wire | §14.2 | 2b | R3, L1 | L |
| R5 | Session resume — session outlives the connection, resume by ID, **content-hash conflict check before replay** | §14.6 | 2 | R1, F10 | M |
| R6 | Name-index replication and batched file open — one round trip to a fully decorated editor | §14.5 | 2b | R2, N2 | S |
| R7 | Remote terminal and tasks — PTY bytes forwarded raw | §14.4 | 2a | R2, D4 | S |

## X — Extensibility

| ID | Feature | Source | Stage | Deps | Size |
|---|---|---|---|---|---|
| X1 | Declarative pack loader — manifest plus assets: grammar, queries, server and adapter definitions, formatter | §17.5 | 1 | F5 | M |
| X5 | Keymap and theme packs — the IntelliJ keymap shipped as a data file | §9.10, §17.5 | 1 | F7, F9 | S |
| X2 | Pack manager UI — browse, install from file, enable/disable, permissions, update | mock Plugins | 4 | X1 | M |
| X3 | WASM extension host — sandboxed runtime, capability permissions declared up front, per-extension metering, **kill-on-hang** | §17.5 | 4 | F11 | L |
| X4 | Declarative UI contributions — panels, status items, tree views, commands as data; extensions never draw | §17.5 | 4 | X3, S2 | M |

## S — Shell, chrome, observability

| ID | Feature | Source | Stage | Deps | Size |
|---|---|---|---|---|---|
| S1 | Window chrome — project switcher, run-configuration selector, run/debug buttons, host switcher | mock Chrome | 1 | F7 | S |
| S2 | Activity rail and tool windows — six rail destinations, collapse, left/right placement | mock | 1 | F7 | M |
| S3 | Bottom dock — Terminal / Debug / Problems / Resources tabs with badges | mock | 2 | S2 | S |
| S4 | Status bar — host, branch, indexing progress, running servers, latency readout, cursor position, encoding | mock | 1 | — | S |
| S5 | Resource panel — per-server memory and CPU against the budget, stop/restart controls | §17.1, mock | 1 | L1 | S |
| S6 | Latency HUD — rolling keystroke→paint histogram, p50/p99 against budget | §16, mock | 0–1 | F6 | S |
| S7 | Indexing progress banner — percentage, and the explicit "every feature stays available" claim | §9.5, mock | 1 | L7 | S |

---

## Not features

Recorded so they are never specified by accident. Principle IX governs the first two.

| Item | Why not |
|---|---|
| "Spec pins" annotation toggle | Mockup scaffolding — it labels the mock with § references. Explicitly excluded by Principle IX. |
| `payments-platform`, `OrderService.java`, sample diagnostics and terminal output | Placeholder content in the mock, not product. Principle IX. |
| Public plugin marketplace / registry | §2 non-goal for v1. Follows X3, not part of it. |
| Bespoke per-language semantic engine (PSI equivalent) | §2 non-goal. The LSP ceiling in §17.4 is accepted deliberately. |
| AI features | §2 non-goal for v1. |

## Gaps — in neither document, decide before they get expensive

| Gap | Why it matters now |
|---|---|
| **Accessibility** | A custom GPU renderer has no native accessibility tree; screen-reader support must be built deliberately (NSAccessibility / UIAutomation / AT-SPI). §17.3 already flags IME as *"must be designed in, not added"* — this is structurally identical and appears nowhere in 727 + 407 lines. Belongs in F3 if it is in v1 at all. |
| **Agent binary trust** | R1 pushes an executable to a remote machine over SSH. Principle VIII sandboxes extensions carefully; the agent has no equivalent clause. |
| **Third-party code intake** | Grammars, packs, and crates arrive from outside. No vetting, pinning, or licensing policy — and tree-sitter grammars are individually-licensed C libraries you would be bundling. |
| **Telemetry / crash reporting** | Currently absent by default rather than by decision. |
| Database tools, HTTP client, Docker/k8s | Named in §17.4's IntelliJ layer-one list but in no roadmap stage. The mock shows Docker only as a *WASM extension card* — suggesting they ship as extensions post-X3, not as core. Worth confirming. |

## The decisions that are actually open

Everything else is dependency-ordered. These are the real choices:

1. **N6 structural search has no roadmap stage.** The mock gives it a full approved modal, so
   Principle IX governs how it looks — but §17.4 mentions it in one sentence and §19.2 never
   schedules it. It is also cheap (F5 is its only dependency) and is called out as *"one of the few
   places Vulcan can be richer than most competitors."* Early differentiator, or Stage 3+?
2. **Accessibility in v1 or not.** Cheap to design into F3 now, very expensive to retrofit later.
3. **How much of Stage 0 to specify.** F1/F2/F3/F5/E1/F6 could be one spec ("the editing surface")
   or six. One keeps the vertical slice honest; six give finer task breakdown.
4. **X1 declarative packs at Stage 1, as §19.2 says?** It is listed under milestone 1 because
   Vulcan's own Java pack ships through it — meaning the pack format must be designed before the
   first language server is wired up, not after.
5. **V7 merge editor is pulled earlier than Stage 3 by R5.** Reconnect conflict resolution needs
   it at Stage 2. Either build it early or give R5 a degraded conflict path.
