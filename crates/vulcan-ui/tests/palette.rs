//! T080: palette filtering, over the prototype's own fixed list.

use vulcan_ui::palette::{count_label, matching, Mode, Row};

fn names(rows: Vec<Row>) -> Vec<&'static str> {
    rows.into_iter().map(|row| row.name).collect()
}

#[test]
fn an_empty_query_shows_every_row() {
    // The palette opens with no query, so this is the state a user sees first.
    let all = Mode::Files.rows();
    assert_eq!(matching(all.clone(), "").len(), all.len());
    assert_eq!(matching(all.clone(), "   ").len(), all.len());
}

#[test]
fn matching_ignores_case() {
    assert_eq!(names(matching(Mode::Files.rows(), "PRICING.GO")), vec!["pricing.go"]);
}

#[test]
fn a_query_matches_the_detail_line_as_well_as_the_name() {
    // "docs/adr/" is a path, not a filename; matching only names would miss it.
    assert_eq!(names(matching(Mode::Files.rows(), "docs/adr")), vec!["orders-ordering.md"]);
}

#[test]
fn a_query_matching_nothing_yields_nothing_rather_than_everything() {
    // A filter that falls back to the full list on no match is worse than an
    // empty result: it reports success for a search that failed.
    assert!(matching(Mode::Files.rows(), "zzz-no-such-symbol").is_empty());
}

#[test]
fn the_row_order_of_the_prototype_is_preserved() {
    let filtered = names(matching(Mode::Files.rows(), "order"));
    assert_eq!(
        filtered,
        vec![
            "OrderService.java",
            "OrderConfirmed.java",
            "order_service_test.go",
            "OrderService.confirm()",
            "orders-ordering.md",
        ]
    );
}

#[test]
fn commands_are_a_separate_list_from_files() {
    assert_eq!(names(matching(Mode::Commands.rows(), "extract")), vec!["Extract method"]);
    assert!(matching(Mode::Files.rows(), "extract").is_empty());
}

#[test]
fn structural_searches_the_same_entries_as_files() {
    // The prototype's ternary is `palette==='commands' ? cmds : files`, so
    // structural shares the file list rather than having one of its own.
    assert_eq!(names(Mode::Structural.rows()), names(Mode::Files.rows()));
}

#[test]
fn the_count_reads_as_shown_of_total() {
    let all = Mode::Files.rows();
    let shown = matching(all.clone(), "order");
    assert_eq!(count_label(shown.len(), all.len()), "5 of 7");
}
