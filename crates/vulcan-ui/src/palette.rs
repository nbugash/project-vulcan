//! The palette's rows and how a query narrows them.
//!
//! Content and filtering come from the prototype: the same seven entries, and
//! the same rule, which is a case-insensitive substring over both the name and
//! the detail line. Matching on the detail as well as the name is what lets
//! "orders" find a file by its path rather than only by its filename.

use crate::icons::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub glyph: &'static str,
    pub name: &'static str,
    pub detail: &'static str,
    pub shortcut: &'static str,
    pub tint: Tint,
}

/// Which colour a row's glyph takes. Named rather than numeric so the rows stay
/// free of design values, which the off-token lint rejects in source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tint {
    Accent,
    Go,
    Warning,
    Modified,
}

/// `paletteModes` in the prototype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Files,
    Commands,
    Structural,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Files => "Files and Symbols",
            Mode::Commands => "Commands",
            Mode::Structural => "Structural",
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Mode::Files => Icon::FILE_CODE,
            Mode::Commands => Icon::COMMAND,
            Mode::Structural => Icon::TREE_STRUCTURE,
        }
    }

    pub fn rows(self) -> Vec<Row> {
        match self {
            Mode::Files | Mode::Structural => files(),
            Mode::Commands => commands(),
        }
    }
}

fn files() -> Vec<Row> {
    vec![
        row(Icon::FILE_CODE, "OrderService.java", "services/orders/src/main/java/…", "⏎", Tint::Accent),
        row(Icon::FILE_CODE, "OrderConfirmed.java", "services/orders/src/main/java/events/", "", Tint::Accent),
        row(Icon::FILE_CODE, "pricing.go", "services/pricing/", "", Tint::Go),
        row(Icon::FILE_CODE, "order_service_test.go", "services/pricing/internal/", "", Tint::Go),
        row(Icon::GEAR_SIX, "vulcan.toml", "packs/ — language pack manifest", "", Tint::Warning),
        row(Icon::FUNCTION, "OrderService.confirm()", "symbol · from tree-sitter tags", "", Tint::Accent),
        row(Icon::FILE_TEXT, "orders-ordering.md", "docs/adr/", "", Tint::Modified),
    ]
}

fn commands() -> Vec<Row> {
    vec![
        row(Icon::ARROWS_CLOCKWISE, "Reload language pack", "packs/vulcan.toml — no restart", "⌘⇧R", Tint::Accent),
        row(Icon::POWER, "Restart language server", "jdtls · 1.14 GB", "", Tint::Accent),
        row(Icon::TREE_STRUCTURE, "Structural search and replace", "tree-sitter query across packs", "⌘⇧S", Tint::Accent),
        row(Icon::CLOCK_COUNTER_CLOCKWISE, "Toggle local history", "per-file rope snapshots", "", Tint::Accent),
        row(Icon::CLOUD, "Attach to remote host", "build-01.euw1 · SSH", "⌘⇧H", Tint::Accent),
        row(Icon::GAUGE, "Show frame timings", "keystroke → paint p99", "", Tint::Accent),
        row(Icon::SCISSORS, "Extract method", "jdtls refactoring", "⌘⌥M", Tint::Accent),
    ]
}

fn row(
    glyph: &'static str,
    name: &'static str,
    detail: &'static str,
    shortcut: &'static str,
    tint: Tint,
) -> Row {
    Row { glyph, name, detail, shortcut, tint }
}

/// An empty or whitespace-only query matches everything, which is what the
/// palette shows when it opens.
pub fn matching(rows: Vec<Row>, query: &str) -> Vec<Row> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return rows;
    }
    rows.into_iter()
        .filter(|row| {
            row.name.to_lowercase().contains(&needle)
                || row.detail.to_lowercase().contains(&needle)
        })
        .collect()
}

/// `paletteCount` in the prototype: "4 of 7".
pub fn count_label(shown: usize, total: usize) -> String {
    format!("{shown} of {total}")
}
