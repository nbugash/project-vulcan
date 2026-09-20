//! Gate 8, triage: discrepancies must be decided, and referenced by link.
//!
//! FR-025: a discrepancy left `Untriaged` at review time is a blocking finding.
//! FR-027: a design document links to a discrepancy rather than restating it,
//! because a restatement is a copy that stops tracking the original.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The template is the shape of a discrepancy, not one of them.
const TEMPLATE: &str = "TEMPLATE.md";

/// The fixed headings of a discrepancy document. A design document reproducing
/// one has restated the body instead of linking to it.
const BODY_HEADINGS: [&str; 3] =
    ["## What we are trying to match", "## Why we cannot match it", "## Alternatives"];

pub struct Discrepancy {
    pub id: String,
    pub path: String,
    pub status: Status,
    pub backlog_entry: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Untriaged,
    Accepted,
    BacklogEntry,
    Unreadable,
}

pub fn read_all(dir: &Path) -> io::Result<Vec<Discrepancy>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "md")
                && path.file_name().is_some_and(|name| name != TEMPLATE)
        })
        .collect();
    paths.sort();

    paths
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path)?;
            Ok(Discrepancy {
                id: path.file_stem().unwrap_or_default().to_string_lossy().into_owned(),
                path: path.to_string_lossy().replace('\\', "/"),
                status: status_of(&text),
                backlog_entry: field(&text, "**Backlog entry**:")
                    .filter(|value| value.starts_with('F')),
            })
        })
        .collect()
}

fn status_of(text: &str) -> Status {
    match field(text, "**Status**:").as_deref() {
        Some("Accepted") => Status::Accepted,
        Some("BacklogEntry") => Status::BacklogEntry,
        Some("Untriaged") => Status::Untriaged,
        // A status line still offering every option is the unedited template.
        _ => Status::Unreadable,
    }
}

fn field(text: &str, label: &str) -> Option<String> {
    let value = text.lines().find_map(|line| line.trim().strip_prefix(label))?.trim();
    (!value.is_empty() && !value.contains('|')).then(|| value.to_string())
}

/// Design documents that restate a discrepancy rather than link to it, or that
/// name one without linking it.
pub fn restatements(design_dir: &Path, known: &[Discrepancy]) -> io::Result<Vec<String>> {
    let mut findings = Vec::new();
    for path in markdown_under(design_dir)? {
        let text = fs::read_to_string(&path)?;
        let file = path.to_string_lossy().replace('\\', "/");

        if let Some(heading) = BODY_HEADINGS.iter().find(|heading| text.contains(**heading)) {
            findings.push(format!(
                "{file} reproduces a discrepancy's \"{heading}\" section; link the document instead"
            ));
        }
        for discrepancy in known {
            if text.contains(&discrepancy.id) && !text.contains(&discrepancy.path) {
                findings.push(format!(
                    "{file} names {} without linking {}",
                    discrepancy.id, discrepancy.path
                ));
            }
        }
    }
    Ok(findings)
}

fn markdown_under(dir: &Path) -> io::Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    let mut entries: Vec<PathBuf> =
        fs::read_dir(dir)?.filter_map(Result::ok).map(|entry| entry.path()).collect();
    entries.sort();
    for entry in entries {
        if entry.is_dir() {
            found.extend(markdown_under(&entry)?);
        } else if entry.extension().is_some_and(|ext| ext == "md") {
            found.push(entry);
        }
    }
    Ok(found)
}
