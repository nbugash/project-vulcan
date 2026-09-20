//! The sample project the prototype composes.
//!
//! Every string here is read from the signed-off prototype rather than invented:
//! the shell renders `payments-platform`, not Vulcan's own source. Keeping it in
//! one module means the next prototype revision is one file to reconcile, and
//! makes it obvious when something on screen has no origin in the mock.
//!
//! Values the prototype animates — the indexing percentage, the frame histogram —
//! are frozen here at the initial state its own code declares, because an exact
//! pixel comparison cannot have a moving subject.

use crate::icons::Icon;
use crate::tokens::Palette;

pub const PROJECT: &str = "payments-platform";
pub const FILE_COUNT: &str = "12 480 files";
pub const RUN_CONFIG: &str = "PaymentsApp";
pub const BRANCH: &str = "feat/stale-quote-retry";
pub const AHEAD: &str = "↑2";
pub const BEHIND: &str = "↓1";

/// `idx: 0.62` in the prototype's initial state.
pub const INDEXING: f32 = 0.62;
pub const INDEXING_LABEL: &str = "indexing 62%";
pub const INDEXING_BANNER: &str =
    "Indexing 62% — every feature stays available; symbols come from tree-sitter tags until the index lands.";

pub const HOST: &str = "Local machine";
pub const SHELL_PROMPT: &str = "~/payments-platform ❯";
pub const DOCK_TITLE: &str = "Terminal — local";
pub const DOCK_SUBTITLE: &str = "PTY · zsh";
pub const CARET: &str = "137:34";

/// Which file marker a tree row carries, and what the prototype tints it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vcs {
    Modified,
    Added,
}

impl Vcs {
    pub fn marker(self) -> &'static str {
        match self {
            Vcs::Modified => "M",
            Vcs::Added => "A",
        }
    }

    pub fn colour(self) -> u32 {
        match self {
            Vcs::Modified => Palette::warning(),
            Vcs::Added => Palette::success(),
        }
    }
}

pub struct TreeRow {
    pub depth: u8,
    pub name: &'static str,
    pub glyph: &'static str,
    pub vcs: Option<Vcs>,
}

/// `treeSrc` in the prototype. Indentation is `10 + depth * 13` pixels.
pub fn tree() -> Vec<TreeRow> {
    let row = |depth, name, glyph, vcs| TreeRow { depth, name, glyph, vcs };
    vec![
        row(0, PROJECT, Icon::CUBE, None),
        row(1, "services", Icon::FOLDER, None),
        row(2, "orders", Icon::PACKAGE, None),
        row(3, "OrderService.java", Icon::FILE_CODE, Some(Vcs::Modified)),
        row(3, "OrderRepository.java", Icon::FILE_CODE, None),
        row(3, "OrderConfirmed.java", Icon::FILE_CODE, None),
        row(2, "pricing", Icon::PACKAGE, None),
        row(3, "pricing.go", Icon::FILE_CODE, Some(Vcs::Modified)),
        row(3, "pricing_test.go", Icon::FILE_CODE, None),
        row(1, "packs", Icon::FOLDER, None),
        row(2, "vulcan.toml", Icon::GEAR_SIX, Some(Vcs::Added)),
        row(2, "go.toml", Icon::GEAR_SIX, None),
        row(1, "deploy", Icon::FOLDER, None),
        row(1, "README.md", Icon::FILE_TEXT, None),
    ]
}

pub const SELECTED_FILE: &str = "OrderService.java";

pub struct Tab {
    pub name: &'static str,
    pub glyph: &'static str,
    pub tint: TabTint,
    pub dirty: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TabTint {
    Accent,
    Go,
    Toml,
}

impl TabTint {
    pub fn colour(self) -> u32 {
        match self {
            TabTint::Accent => Palette::accent(),
            TabTint::Go => Palette::go_tint(),
            TabTint::Toml => Palette::warning(),
        }
    }
}

/// `tabMeta` in the prototype: icon, tint, and whether the buffer is dirty.
pub fn tabs() -> Vec<Tab> {
    vec![
        Tab { name: "OrderService.java", glyph: Icon::FILE_CODE, tint: TabTint::Accent, dirty: true },
        Tab { name: "pricing.go", glyph: Icon::FILE_CODE, tint: TabTint::Go, dirty: false },
        Tab { name: "vulcan.toml", glyph: Icon::GEAR_SIX, tint: TabTint::Toml, dirty: true },
    ]
}

/// `crumbSrc[0]`, the trail for the active tab.
pub fn breadcrumbs() -> [&'static str; 5] {
    ["services", "orders", "src/main/java", "OrderService", "confirm()"]
}

pub const BREADCRUMB_NOTE: &str = "from tree-sitter tags";

/// How a run of code is coloured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syntax {
    Plain,
    Keyword,
    Type,
    Function,
    Punctuation,
    Comment,
    Inlay,
}

impl Syntax {
    pub fn colour(self) -> u32 {
        match self {
            Syntax::Plain => Palette::syn_identifier(),
            Syntax::Keyword => Palette::syn_keyword(),
            Syntax::Type => Palette::syn_type(),
            Syntax::Function => Palette::syn_function(),
            Syntax::Punctuation => Palette::syn_punctuation(),
            Syntax::Comment => Palette::syn_comment(),
            Syntax::Inlay => Palette::inlay(),
        }
    }
}

pub struct Line {
    pub number: u32,
    /// The prototype marks edited lines with a 3px bar in the gutter.
    pub changed: bool,
    pub breakpoint: bool,
    pub current: bool,
    pub spans: Vec<(Syntax, &'static str)>,
}

/// The buffer as the prototype paints it: `OrderService.confirm()`, lines
/// 126–145, with the language server's inlay hints resolved.
pub fn buffer() -> Vec<Line> {
    use Syntax::{Comment, Function, Inlay, Keyword, Plain, Punctuation, Type};
    let line = |number, changed, breakpoint, current, spans| Line {
        number,
        changed,
        breakpoint,
        current,
        spans,
    };
    vec![
        line(126, false, false, false, vec![(Comment, "    // §9.1 anchors keep these positions valid across edits")]),
        line(127, true, false, false, vec![(Keyword, "@Transactional")]),
        line(128, true, false, false, vec![
            (Keyword, "public "), (Type, "Receipt "), (Function, "confirm"), (Punctuation, "("),
            (Type, "OrderId"), (Plain, " id"), (Punctuation, ", "), (Type, "PaymentRef"),
            (Plain, " ref"), (Punctuation, ") {"),
        ]),
        line(129, false, false, false, vec![
            (Keyword, "    var"), (Plain, " order"), (Inlay, "  : Order"), (Punctuation, " = "),
            (Plain, "orders"), (Punctuation, "."), (Function, "require"), (Punctuation, "("),
            (Plain, "id"), (Punctuation, ");"),
        ]),
        line(130, false, false, false, vec![
            (Plain, "    order"), (Punctuation, "."), (Function, "assertConfirmable"), (Punctuation, "();"),
        ]),
        line(131, false, false, false, vec![(Plain, " ")]),
        line(132, true, false, false, vec![
            (Keyword, "    var"), (Plain, " quote"), (Inlay, "  : Quote"), (Punctuation, " = "),
            (Plain, "pricing"), (Punctuation, "."), (Function, "quote"), (Punctuation, "("),
            (Plain, "order"), (Punctuation, "."), (Function, "lines"), (Punctuation, "());"),
        ]),
        line(133, true, false, false, vec![
            (Keyword, "    if"), (Punctuation, " ("), (Plain, "quote"), (Punctuation, "."),
            (Function, "isStale"), (Punctuation, "()) {"),
        ]),
        line(134, true, false, false, vec![
            (Keyword, "        return "), (Type, "Receipt"), (Punctuation, "."), (Function, "retry"),
            (Punctuation, "("), (Plain, "quote"), (Punctuation, "."), (Function, "ttl"), (Punctuation, "());"),
        ]),
        line(135, true, false, false, vec![(Punctuation, "    }")]),
        line(136, false, false, false, vec![(Plain, " ")]),
        line(137, false, false, true, vec![
            (Keyword, "    var"), (Plain, " charge = payments"), (Punctuation, "."),
            (Function, "capture"), (Punctuation, "("), (Plain, "ref"), (Punctuation, ", "),
            (Plain, "quote"), (Punctuation, "."), (Function, "total"), (Punctuation, "());"),
        ]),
        line(138, false, false, false, vec![
            (Plain, "    order"), (Punctuation, "."), (Function, "confirm"), (Punctuation, "("),
            (Plain, "charge"), (Punctuation, "."), (Function, "id"), (Punctuation, "(), "),
            (Plain, "clock"), (Punctuation, "."), (Function, "now"), (Punctuation, "());"),
        ]),
        line(139, false, false, false, vec![(Plain, " ")]),
        line(140, false, true, false, vec![
            (Plain, "    events"), (Punctuation, "."), (Function, "publish"), (Punctuation, "("),
            (Keyword, "new "), (Type, "OrderConfirmed"), (Punctuation, "("), (Plain, "order"),
            (Punctuation, "."), (Function, "id"), (Punctuation, "(), "), (Plain, "charge"),
            (Punctuation, "."), (Function, "id"), (Punctuation, "()));"),
        ]),
        line(141, false, false, false, vec![
            (Keyword, "    return "), (Type, "Receipt"), (Punctuation, "."), (Function, "of"),
            (Punctuation, "("), (Plain, "order"), (Punctuation, ", "), (Plain, "charge"), (Punctuation, ");"),
        ]),
        line(142, false, false, false, vec![(Punctuation, "}")]),
        line(143, false, false, false, vec![(Plain, " ")]),
        line(144, false, false, false, vec![
            (Comment, "    /* pricing.quote is remote — the editor never waits on it (§13) */"),
        ]),
        line(145, false, false, false, vec![
            (Keyword, "    private void "), (Function, "assertLimits"), (Punctuation, "("),
            (Type, "Order"), (Plain, " order"), (Punctuation, ") {"),
        ]),
    ]
}

/// How a terminal line is coloured, as `term` declares it.
#[derive(Debug, Clone, Copy)]
pub enum Output {
    Muted,
    Text,
    Task,
    Pass,
}

impl Output {
    pub fn colour(self) -> u32 {
        match self {
            Output::Muted => Palette::muted(),
            Output::Text => Palette::text(),
            Output::Task => Palette::syn_string(),
            Output::Pass => Palette::success(),
        }
    }
}

pub fn terminal() -> Vec<(Output, &'static str)> {
    use Output::{Muted, Pass, Task, Text};
    vec![
        (Muted, "ssh build-01.euw1 — connected in 240 ms, agent 0.4.2"),
        (Text, "~/payments-platform ❯ ./gradlew :orders:test --tests OrderServiceTest"),
        (Task, "> Task :orders:compileJava"),
        (Task, "> Task :orders:test"),
        (Pass, "OrderServiceTest > confirmsWhenQuoteFresh() PASSED"),
        (Pass, "OrderServiceTest > retriesWhenQuoteStale() PASSED"),
        (Pass, "BUILD SUCCESSFUL in 6s"),
        (Muted, "4 actionable tasks: 2 executed, 2 up-to-date"),
    ]
}

/// `dockTabs`, with the badge each carries in the initial state.
pub fn dock_tabs() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (Icon::TERMINAL_WINDOW, "Terminal", ""),
        (Icon::BUG, "Debug", ""),
        (Icon::WARNING_CIRCLE, "Problems", "1"),
        (Icon::GAUGE, "Resources", ""),
    ]
}

pub struct Completion {
    pub name: &'static str,
    pub signature: &'static str,
    pub kind: &'static str,
    pub source: &'static str,
    /// The prototype dims an entry resolved from a lower-confidence source.
    pub certain: bool,
}

/// `compAll` in the prototype.
pub fn completions() -> Vec<Completion> {
    let entry = |name, signature, kind, source, certain| Completion {
        name,
        signature,
        kind,
        source,
        certain,
    };
    vec![
        entry("capture", "(PaymentRef ref, Money amount) → Charge", "M", "jdtls", true),
        entry("captureAsync", "(PaymentRef ref, Money amount) → CompletableFuture<Charge>", "M", "jdtls", true),
        entry("capturePartial", "(PaymentRef ref, Money amount, Split split) → Charge", "M", "jdtls", true),
        entry("capturedAt", "() → Instant", "M", "jdtls", true),
        entry("CAPTURE_TIMEOUT", "Duration — static field", "F", "jdtls", true),
        entry("capture_ref", "payments.capture_ref — column", "C", "sql pack", false),
    ]
}

pub const COMPLETION_SOURCE: &str = "jdtls";
pub const COMPLETION_TIMING: &str = "41 ms";
pub const COMPLETION_QUERY: &str = "type to filter — watch the request get cancelled";
pub const COMPLETION_FOOTER: &str = "jdtls + sql pack, merged by source";

/// The documentation panel beside the selected entry.
pub const DOC_SIGNATURE: [&str; 3] = ["Charge capture(", "    PaymentRef ref,", "    Money amount)"];
pub const DOC_BODY: [&str; 2] = [
    "Captures an authorised payment. Idempotent on ref; throws ExpiredAuthorization after 7 days.",
    "Documentation resolved on selection — a separate, cancellable request.",
];

/// Language servers in the status bar, with their resident size.
pub fn servers() -> Vec<(&'static str, &'static str)> {
    vec![("jdtls", "1.14 GB"), ("gopls", "214 MB")]
}

pub const ENCODING: [&str; 3] = ["LF", "UTF-8", "4 spaces"];

/// `lat: Array.from({length:34},(_,i)=>2.4+((i*7)%11)/4)` — the frame histogram,
/// seeded rather than sampled so the reproduction is deterministic.
pub fn frame_histogram() -> Vec<f32> {
    (0..34).map(|i: u32| 2.4 + ((i * 7) % 11) as f32 / 4.0).collect()
}

pub const LATENCY_TITLE: &str = "Keystroke → paint";
pub const LATENCY_BUDGET: &str = "budget 8.3ms";
pub const LATENCY_NOTE: &str =
    "One 120 Hz frame is the SLO (§16). Regressions fail CI, not review.";

/// p50 and p99 over the seeded histogram, which is what the prototype shows
/// before its first tick: median 3.65 and maximum 4.9 of `2.4 + k/4`.
pub const LATENCY_P50: &str = "p50 3.6 ms";
pub const LATENCY_P99: &str = "p99 4.9 ms";
pub const STATUS_LATENCY: &str = "p99 4.9 ms";
