//! T100 / FR-016 / FR-033: every state the shell reaches is either depicted by
//! the prototype or recorded as a prototype extension.

use std::fs;
use std::path::PathBuf;

use vulcan_ui::states::{reachable, Depiction};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The self-contained prototype under `mockups/`. The rule for *which* file
/// that is lives in the domain, so this and the adapter cannot drift apart —
/// `vulcan-ui` may not depend on `vulcan-adapters`, that being the sibling edge
/// gate 1 exists to reject.
fn prototype() -> String {
    let mut found: Vec<String> = fs::read_dir(repo().join("mockups"))
        .expect("mockups directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "html"))
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter(|text| vulcan_domain::prototype::is_self_contained(text))
        .collect();
    assert_eq!(found.len(), 1, "expected exactly one self-contained prototype");
    found.remove(0)
}

/// The `### ` headings under `## Prototype extensions` in the design document.
fn recorded() -> Vec<String> {
    let design = fs::read_to_string(repo().join("specs/001-engineering-baseline/design.md"))
        .expect("design document");
    let section = design
        .split("## Prototype extensions")
        .nth(1)
        .expect("design document has a prototype extensions section");
    section
        .lines()
        .take_while(|line| !line.starts_with("## "))
        .filter_map(|line| line.strip_prefix("### "))
        .map(|heading| heading.trim().to_string())
        .collect()
}

#[test]
fn every_state_the_prototype_depicts_names_evidence_that_is_really_there() {
    // Without this the list is a set of claims about itself: anything could
    // declare itself depicted and nothing would check.
    let prototype = prototype();
    let missing: Vec<&str> = reachable()
        .iter()
        .filter_map(|state| match state.depiction {
            Depiction::Prototype(evidence) if !prototype.contains(evidence) => Some(state.name),
            _ => None,
        })
        .collect();
    assert!(missing.is_empty(), "claims the prototype depicts, but it does not: {missing:?}");
}

#[test]
fn every_undepicted_state_is_recorded_in_the_design_document() {
    let recorded = recorded();
    let unrecorded: Vec<&str> = reachable()
        .iter()
        .filter_map(|state| match state.depiction {
            Depiction::Extension(heading) if !recorded.iter().any(|r| r == heading) => {
                Some(state.name)
            }
            _ => None,
        })
        .collect();
    assert!(
        unrecorded.is_empty(),
        "reachable but not recorded as a prototype extension: {unrecorded:?}"
    );
}

#[test]
fn no_extension_is_recorded_for_a_state_that_no_longer_exists() {
    // A record left behind after its state is removed is as misleading as a
    // missing one: review trusts the list to be current.
    let declared: Vec<&str> = reachable()
        .iter()
        .filter_map(|state| match state.depiction {
            Depiction::Extension(heading) => Some(heading),
            _ => None,
        })
        .collect();
    let stale: Vec<String> =
        recorded().into_iter().filter(|r| !declared.contains(&r.as_str())).collect();
    assert!(stale.is_empty(), "recorded, but no longer reachable: {stale:?}");
}
