//! T108 / T109: discrepancy triage and reference-by-link.

use std::fs;
use std::path::PathBuf;

use vulcan_adapters::reporting::discrepancies::{read_all, restatements, Status};

fn sandbox(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vulcan-disc-{}-{label}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write(dir: &PathBuf, name: &str, status: &str, backlog: &str) {
    fs::write(
        dir.join(name),
        format!(
            "# {name}\n\n**Status**: {status}\n**Backlog entry**: {backlog}\n\n\
             ## What we are trying to match\nx\n\n## Why we cannot match it\ny\n\n## Alternatives\n1. z\n"
        ),
    )
    .expect("write");
}

#[test]
fn a_decided_document_reports_its_status() {
    let dir = sandbox("decided");
    write(&dir, "001-a.md", "Accepted", "—");
    write(&dir, "002-b.md", "BacklogEntry", "F012");

    let found = read_all(&dir).expect("read");
    assert_eq!(found[0].status, Status::Accepted);
    assert_eq!(found[1].status, Status::BacklogEntry);
    assert_eq!(found[1].backlog_entry.as_deref(), Some("F012"));
}

#[test]
fn the_unedited_template_line_is_not_a_decision() {
    // "Untriaged | Accepted | BacklogEntry" is the template offering choices,
    // not someone having made one.
    let dir = sandbox("template");
    write(&dir, "001-a.md", "Untriaged | Accepted | BacklogEntry", "—");
    assert_eq!(read_all(&dir).expect("read")[0].status, Status::Unreadable);
}

#[test]
fn the_template_itself_is_not_a_discrepancy() {
    let dir = sandbox("skip-template");
    write(&dir, "TEMPLATE.md", "Untriaged", "—");
    write(&dir, "001-a.md", "Accepted", "—");

    let found = read_all(&dir).expect("read");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, "001-a");
}

#[test]
fn a_missing_directory_is_nothing_to_judge_rather_than_a_failure() {
    let dir = std::env::temp_dir().join("vulcan-disc-absent-never-created");
    assert!(read_all(&dir).expect("read").is_empty());
}

#[test]
fn a_design_document_reproducing_a_discrepancy_body_is_a_finding() {
    let discrepancies = sandbox("restate-src");
    write(&discrepancies, "001-a.md", "Accepted", "—");
    let known = read_all(&discrepancies).expect("read");

    let designs = sandbox("restate-designs");
    fs::write(
        designs.join("shell.md"),
        "# Shell\n\n## Why we cannot match it\nthe same words again\n",
    )
    .expect("write");

    let findings = restatements(&designs, &known).expect("scan");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].contains("Why we cannot match it"), "{findings:?}");
}

#[test]
fn naming_a_discrepancy_without_linking_it_is_a_finding() {
    let discrepancies = sandbox("link-src");
    write(&discrepancies, "001-a.md", "Accepted", "—");
    let known = read_all(&discrepancies).expect("read");

    let designs = sandbox("link-designs");
    fs::write(designs.join("shell.md"), "# Shell\n\nSee 001-a for the typeface decision.\n")
        .expect("write");

    let findings = restatements(&designs, &known).expect("scan");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].contains("without linking"), "{findings:?}");
}

#[test]
fn a_design_document_that_links_the_path_is_clean() {
    let discrepancies = sandbox("clean-src");
    write(&discrepancies, "001-a.md", "Accepted", "—");
    let known = read_all(&discrepancies).expect("read");

    let designs = sandbox("clean-designs");
    fs::write(
        designs.join("shell.md"),
        format!("# Shell\n\nSee [the typeface decision]({}).\n", known[0].path),
    )
    .expect("write");

    assert!(restatements(&designs, &known).expect("scan").is_empty());
}
