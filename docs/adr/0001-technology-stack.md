# ADR 0001 — Technology stack

- **Status:** Proposed (pending Stage 0 spike)
- **Date:** 2026-09-16
- **Governs:** the whole implementation
- **Constrained by:** `speckit-constitution.md` → *Technology and Platform Constraints*
- **Evidence:** `vulcan-system-design.md` §4–§6, §14.4, §17.1–§17.3, §19.1, §19.3

## Context

The constitution fixes the *properties* an implementation stack must have — no GC pause on the
keystroke path, deterministic footprint within Principle VII, GPU rendering with damage tracking
on three platforms, an incremental data-driven syntax layer, LSP/DAP process supervision, and
a compact binary wire format.
It deliberately does not name the stack, because `vulcan-system-design.md` §19.3 anticipates
that some of these choices will be revisited and a constitutional amendment is the wrong
instrument for that.

This ADR records the selection.

## Decision

| Layer | Selection | Confidence |
|---|---|---|
| Language and runtime | **Rust** | High — settled |
| UI framework | **Floem / wgpu**; GPUI only if a spike proves its Windows support adequate | **Low — the main open question** |
| Text buffer | Rope with summary nodes — Zed's SumTree design, or `ropey` plus an anchor layer | Medium — design settled, crate choice open |
| Syntax | tree-sitter, in-process, on a background thread | High — settled |
| Semantics and debugging | LSP + DAP only; no bespoke engine. Java via `jdtls` and the Java debug adapter | High — settled |
| Remote transport | Vulcan protobuf frames over SSH (`ssh host vulcan-agent --stdio`), agent binary pushed on first connect | Medium — §14.4 channel layout is Vulcan's own and unvalidated until Stage 2 |
| Extensions | Declarative packs (TOML + assets) first; WASM via `wasmtime` with capability permissions later | High — settled |

## Rationale

**Rust.** The fastest languages for an editor core are C, C++, Rust, and Zig; the measured
difference between them for this workload is noise. Rust wins on memory safety, ecosystem, and
the existence of two open-source IDE-grade reference implementations to read — Zed (GPUI) and
Lapce (Floem). The 12 GB budget in §17.1 rules out a JVM core: a JVM IDE process starts near
700 MB–1 GB before a project is loaded and then competes for RAM with the JVM language servers
it must spawn. Notably this is *not* a GC-pause argument — Generational ZGC in Java 25 holds
pauses to 0.1–0.5 ms, well inside a frame. It is a footprint argument.

**Floem over GPUI.** Floem is editor-purpose-built and gentler to work with; GPUI is more
polished but pre-1.0, macOS-first, with Windows support still maturing. Windows viability is the
deciding factor and is a Stage 0 deliverable. Both are pre-1.0, so either choice means tracking
a moving target.

**Rope with summary nodes.** O(log n) edits and coordinate conversion at any file size, plus
cheap persistent snapshots via copy-on-write — which is the enabling mechanism for Principle II,
not merely a performance detail. A piece table fragments under scattered edits; a gap buffer is
O(n) when the gap moves.

**tree-sitter as the syntax layer.** The constitution requires an incremental, error-tolerant
syntax layer driven by per-language data rather than host code, able to supply highlighting,
folding, indentation, and in-file symbols with no index and no language server. tree-sitter is
the only mature implementation of exactly that shape: grammars compile to C libraries, an edit
patches and re-parses only the affected region (typically well under a millisecond per
keystroke), and its query files — `highlights`, `folds`, `indents`, `locals`, `injections`,
`tags` — cover every capability the constraint names, including the cold-index symbol fallback
Principle III depends on and the embedded-language support (SQL in a Java string) that a regex
grammar cannot express.

The alternatives fail the constraint rather than merely losing to it. TextMate regex grammars,
as used by VS Code, are non-incremental and carry no structure, so they cannot supply folds,
indents, or tags. A hand-written lexer/parser per language, as in IntelliJ's PSI, is deeper but
is host code per language — which violates "adding a language MUST cost zero host code" in
Principle VIII outright.

Substituting tree-sitter therefore means demonstrating a replacement that satisfies all six
clauses of the constraint. It is settled in practice, but it is recorded here rather than in the
constitution so that the *property* governs and the *library* does not.

**LSP + DAP only.** Accepts the ceiling documented in §17.4: an LSP-based IDE is as deep as its
servers and will not match IntelliJ's deepest cross-file refactorings. The trade buys
language-agnostic breadth at zero host code per language.

**protobuf over SSH.** SSH gives authentication, encryption, and host trust from the user's
existing configuration for free, with no daemon and no listening port. Binary framing rather
than forwarded JSON-RPC keeps large-payload decoding off the laptop, which is §4.2's xi-editor
lesson applied one layer up.

## Alternatives rejected

These follow from the constitution's principles rather than from this ADR, and are listed here
only so the reasoning is in one place. Re-proposing them requires a constitutional amendment.

| Rejected | Why |
|---|---|
| RPC between keystroke and glyph | Violates Principle I. Killed xi-editor; the reason Electron editors lag. |
| Electron / Tauri | Measurably laggier and heavier than native; Tauri reintroduces a web renderer. |
| C or C++ for a presumed ~10% win | No measurable latency advantage over Rust for this workload; costs memory safety and months of solo productivity. |
| Zig | Pre-1.0, no mature GUI toolkit, no editor reference implementation. |
| JVM core (Kotlin + Compose/Skiko) | Fails the §17.1 footprint budget. Retained only as the §19.3 fallback if Rust itself proves untenable. |
| C# / .NET + Avalonia | Credible on the merits; rejected for having no IDE-of-record to learn from and being outside the author's ecosystem. |

## Consequences

- **Compile times on 5 cores will hurt.** Mitigations are expected practice, not optimizations:
  incremental compilation, `mold`/`lld`, the Cranelift backend for debug builds, a well-split
  crate graph, possibly a remote build box. Budget for this.
- **Two pre-1.0 dependencies** (the UI framework, and tree-sitter grammar churn) mean breaking
  changes are routine. §19.2 front-loads Stage 0 specifically to surface this early.
- **Rust is a real learning cost** for an experienced Java engineer, concentrated in the
  ownership model as it applies to UI — Zed has written a whole post on this.
- **The Nocturne design system does not port directly.** `mockups/Vulcan IDE.html` expresses it
  as CSS custom properties with OKLCH-derived ramps; Floem has no CSS. Those tokens must be
  re-expressed as Rust theme data per §9.10. This work is currently unscoped.

## Revisit triggers

From `vulcan-system-design.md` §19.3. Hitting one of these reopens this ADR — and only this
ADR, not the constitution, provided the constitution's stack constraints still hold:

1. Stage 0 exceeds ~2 months or stalls on the UI framework → fall back to Floem if on GPUI; if
   the obstacle is Rust itself, escalate to Kotlin + Compose/Skiko, accepting the §17.1 memory
   compromise. That escalation *does* require a constitutional amendment, because a JVM core
   cannot satisfy the footprint constraint as written.
2. GPUI's Windows support proves inadequate in the spike → Floem, decided by Stage 0 exit.
3. Typing exceeds ~8–10 ms p99 after Principles I and II are applied → this is an architecture
   defect, **not** a stack problem. Fix the hot path. Do not reopen this ADR.
4. Remote typing measurably slower than local → likewise an architecture defect. Find the network
   hop on the hot path. Do not reopen this ADR.
