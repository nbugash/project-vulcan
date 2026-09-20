# T111: reconciling the shell with the updated prototype

**Status**: complete. Every item below is implemented; the fidelity reference has been
re-captured against the current prototype and `compare` reports no difference.

**Date**: 2026-09-19, revised the same day after a second prototype update

The `mockups/` tree was replaced after the shell was built. The design system is unchanged —
`gate-fidelity extract` produces byte-identical tokens — so nothing here is a token change.
What moved is composition and content.

Evidence: `reports/screenshots/prototype-standalone.png` is the current prototype, rendered at
the 1440x900 viewport; `reports/screenshots/F000-engineering-baseline.png` is the shell.
(`prototype-reference.png` is the previous revision, kept for comparison.)

Two items from the first pass are now closed by the prototype itself:

- **The code typeface.** The prototype pins `'JetBrains Mono', ui-monospace, Menlo, monospace`
  and loads the face; a width probe confirms it paints rather than falling back. The product
  already pinned JetBrains Mono, so `reports/mockups-discrepancies/001-code-typeface.md` was
  retired — it described something the product could not match, and the two now agree.
- **Indexing percentage.** The prototype now reads 68%, which is what the shell renders.

The off-token lint already caught and fixed four invented dimensions against this prototype
(the search field's width, the palette's offset and width, the completion popup's anchor). The
items below are what the lint cannot see: placement, presence and content.

## A. Regions in the wrong place or missing

| # | Item | Prototype | Shell today |
|---|------|-----------|-------------|
| A1 | Indexing banner | A full-width row **inside the editor, directly below the breadcrumbs**, reading `Indexing 68% — every feature stays available; symbols come from tree-sitter tags until the index lands` | A strip above the status bar at the window's bottom edge |
| A2 | Latency HUD | A floating panel bottom-right: "Keystroke → paint", a frame histogram, `p50 3.6 ms  p99 4.9 ms  budget 8.3ms`, and a note that one 120 Hz frame is the SLO | Absent entirely |
| A3 | Completion popup | Shows the **documentation side panel** — signature, prose, and a resolution note — the `detail` style | Renders the `compact` style only |
| A4 | Breadcrumb row | Carries `from tree-sitter tags` at its right edge | No right-hand label |
| A5 | Toolbar, right side | Two labelled pills, `Spec pins` and `Local`, then a sliders control | Four unlabelled icons |
| A6 | Editor gutter | A breakpoint dot on line 140, a current-line band, and change marks | Neither breakpoint nor current-line treatment |

## B. Content that is invented rather than the prototype's

| # | Item | Prototype | Shell today |
|---|------|-----------|-------------|
| B1 | Project | `payments-platform`: `services/orders/{OrderService,OrderRepository,OrderConfirmed}.java`, `services/pricing/{pricing.go,pricing_test.go}`, `packs/{vulcan.toml,go.toml}`, `deploy/`, `README.md` | Vulcan's own crates and tools |
| B2 | File count | `12 480 files`, space-separated | `1,284 files` |
| B3 | Editor buffer | `OrderService.confirm()` in Java, lines 126–144 | Rust from this repository |
| B4 | Editor tabs | `OrderService.java`, `pricing.go`, `vulcan.toml` | `main.rs`, `shell.rs`, `Cargo.toml` |
| B5 | Terminal | A Gradle run: `./gradlew :orders:test`, two passing tests, `BUILD SUCCESSFUL in 6s` | Output of this project's own gates |
| B6 | Dock header | `TERMINAL — LOCAL PTY·zsh` | `Terminal — build-01.euw1 PTY · zsh` |
| B7 | Completion entries | `capture`, `captureAsync`, `capturePartial`, `capturedAt`, `CAPTURE_TIMEOUT`, `capture_ref`, each with kind and source pack | Rust symbols |
| B8 | Toolbar selectors | Project `payments-platform`, run configuration `PaymentsApp` | `vulcan`, `PaymentsApp` |
| B9 | Status bar | `Local machine` · `feat/stale-quote-retry ↑2 ↓1` · progress · `indexing 68% · nothing disabled` · `jdtls 1.14 GB` · `gopls 214 MB` · `p99 4.9 ms` · `137:34` · `LF` · `UTF-8` · `4 spaces` | A remote host readout and this project's counters |

## Consequences for work already recorded

- **The remote banner.** The prototype's status bar reads `Local machine`, not a remote host.
  The relocation to the bottom-left was right; the content was invented. This is content
  (B9), not a prototype extension.
- **The latency HUD (A2) is a depicted state the shell cannot currently reach**, so it is the
  opposite of a prototype extension: the prototype composes it and the reproduction omits it.
  `crates/vulcan-ui/src/states.rs` must gain it once built.
- **The fidelity reference is current but premature.** It was captured from today's shell, so
  `compare` passes; it will need re-capturing after every item above, and until then it
  certifies only that the shell has not regressed from itself.

## Sequencing

A1–A6 are structural and independent of each other. B1–B9 are one change: the sample fixture
the shell renders. Doing B first makes A easier to verify, because every region then shows the
same text as the prototype and a difference is visibly a difference rather than a rename.

## The prototype's filename is not part of its identity

This revision also renamed the file: `Vulcan-IDE.html` became
`Vulcan IDE (standalone).html`, alongside a second export, `Vulcan IDE.dc.html`.

Ten places hardcoded the old name, and the rename made `gate-fidelity extract` **pass while
reading no prototype at all** — it needs only the stylesheet and the asset manifest, so it
never noticed the source of truth had gone. That is the failure this suite exists to prevent.

The rule is now a property rather than a name, in `vulcan_domain::prototype`: the prototype is
the document under `mockups/` that fetches nothing outside itself. `Vulcan IDE.dc.html`
references `_ds/`; the standalone export inlines everything, so it is the one the gates read.
If both were self-contained, or neither, the gates refuse and name the candidates rather than
guess.

## What was built

**Content (B1–B9).** The sample project now lives in `crates/vulcan-ui/src/fixture.rs`, read
from the prototype rather than invented: the `payments-platform` tree, its tabs and
breadcrumbs, the Java buffer at lines 126–145 with the language server's inlay hints resolved,
the Gradle terminal session, the completion entries with their kinds and source packs, and the
status bar's host, branch, servers and caret. One module, so the next prototype revision is
one file to reconcile.

**Structure (A1–A6).**

- **A1** The indexing banner moved into the editor, below the breadcrumbs.
- **A2** The latency HUD was built. The prototype gates it on `hud: s.hud || perf === 'hud'`,
  which is the performance-readout prop the shell already carried, so it needed no new state.
- **A3** The completion popup gained its documentation panel, 258px as the prototype declares,
  shown in the detail style.
- **A4** The breadcrumb row carries `from tree-sitter tags` at its right edge.
- **A5** The toolbar's bare glyphs became the prototype's labelled `Spec pins` and `Local`
  pills.
- **A6** The gutter is 52px with a 3px change bar, a breakpoint on line 140 and a band on the
  caret's line.

## Two corrections the work produced

**The syntax palette was wrong.** `Palette::syn_*` mapped the syntax colours onto the neutral
and accent ramps by approximation — keywords resolved to `accent-400` where the prototype
paints `#b5abfc`. The asset manifest names every one of these for its role, so they are now
read from those entries. This was invisible to both gates: the lint checks that a value comes
from the prototype, not that it is used where the prototype uses it, and the comparison
captures its reference from the shell.

**The prototype animates.** Its indexing percentage cycles and its frame histogram is
`Math.random()`. An exact comparison cannot have a moving subject, so the fixture freezes both
at the initial state the prototype's own code declares: `idx: 0.62`, and
`lat: Array.from({length:34},(_,i)=>2.4+((i*7)%11)/4)`, whose median and maximum are the
`p50 3.6 ms` and `p99 4.9 ms` the prototype shows before its first tick. An earlier note in
this document claimed the prototype "now reads 68%" — that was a sampled frame of an
animation, not a change to the mock.
