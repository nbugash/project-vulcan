//! T058: the off-token scanner reads the reproduction's source.

use std::fs;
use std::path::PathBuf;

use vulcan_adapters::tokens::scanner::SourceScanner;

/// Cargo runs tests in one process in parallel, so a directory named only for
/// the process is shared and the tests race each other.
fn sandbox(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vulcan-scanner-{}-{label}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn scan(label: &str, source: &str) -> Vec<(String, usize)> {
    let dir = sandbox(label);
    fs::write(dir.join("sample.rs"), source).expect("sample");

    let found = SourceScanner::new([dir.clone()]).scan().expect("scan");
    fs::remove_dir_all(&dir).ok();
    found.into_iter().map(|literal| (literal.value, literal.line)).collect()
}

#[test]
fn a_colour_literal_is_found_with_its_line() {
    let found = scan("colour", "fn a() {\n    div().bg(rgb(0x1E1F22));\n}\n");
    assert_eq!(found, vec![("#1e1f22".to_string(), 2)]);
}

#[test]
fn a_dimension_is_normalised_to_the_form_the_token_set_uses() {
    // `px(8.0)` and the token `"8px"` are the same value written two ways; the
    // comparison is a string match, so the scanner must do the conversion.
    let found = scan("dimension", "fn a() {\n    div().p(px(8.0)).m(px(10.5));\n}\n");
    assert_eq!(found, vec![("10.5px".to_string(), 2), ("8px".to_string(), 2)]);
}

#[test]
fn a_value_named_in_a_comment_is_not_a_finding() {
    // Documentation describes design decisions; it does not make them.
    let found = scan("comment", "fn a() {\n    // matches rgb(0xff0000) in the prototype\n    ();\n}\n");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn the_generated_token_module_is_not_linted_against_itself() {
    let dir = sandbox("generated");
    fs::write(dir.join("generated_tokens.rs"), "pub const A: u32 = rgb(0x1e1f22);\n").expect("gen");

    let found = SourceScanner::new([dir.clone()]).scan().expect("scan");
    fs::remove_dir_all(&dir).ok();
    assert!(found.is_empty(), "the file that defines the values cannot violate them: {found:?}");
}

#[test]
fn findings_are_ordered_so_the_report_is_stable_across_machines() {
    let dir = sandbox("order");
    fs::write(dir.join("b.rs"), "fn b() { px(2.0); }\n").expect("b");
    fs::write(dir.join("a.rs"), "fn a() { px(1.0); }\n").expect("a");

    let found: Vec<String> = SourceScanner::new([dir.clone()])
        .scan()
        .expect("scan")
        .into_iter()
        .map(|literal| literal.value)
        .collect();
    fs::remove_dir_all(&dir).ok();
    assert_eq!(found, vec!["1px".to_string(), "2px".to_string()]);
}

#[test]
fn the_shell_source_carries_no_value_absent_from_the_prototype() {
    // T081: the same check gate 8 runs, as a test, so a literal introduced in an
    // editor fails before it reaches the gate.
    use vulcan_adapters::tokens::stylesheet::StylesheetAndManifestAdapter;
    use vulcan_app::ports::token_source::TokenSourcePort;
    use vulcan_app::use_cases::extract_tokens::ExtractTokens;
    use vulcan_app::use_cases::lint_off_token::LintOffToken;

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let adapter = StylesheetAndManifestAdapter::discover(&root.join("mockups")).expect("prototype");
    let vocabulary = adapter.read_vocabulary().expect("vocabulary");
    let tokens = ExtractTokens::new(adapter).execute().expect("tokens");
    let found = SourceScanner::new([root.join("crates/vulcan-ui/src")]).scan().expect("scan");

    assert!(!found.is_empty(), "scanning found nothing, so it proves nothing");
    match LintOffToken::execute(&tokens, &vocabulary, &found) {
        vulcan_domain::verdict::GateVerdict::Passed => {}
        other => panic!("{other:?}"),
    }
}
