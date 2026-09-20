//! T011: verdict-to-exit-code mapping.
//!
//! Three codes with distinct meanings. `CouldNotJudge` must never be mistaken
//! for a pass, which is the failure mode where a missing prerequisite silently
//! disables a gate.

use vulcan_domain::verdict::{GateError, GateVerdict};

#[test]
fn judged_and_passed_exits_zero() {
    assert_eq!(GateVerdict::Passed.exit_code(), 0);
}

#[test]
fn judged_and_failed_exits_two() {
    let verdict = GateVerdict::Failed {
        findings: vec!["boundary crossed".to_string()],
    };
    assert_eq!(verdict.exit_code(), 2);
}

#[test]
fn could_not_judge_exits_one() {
    assert_eq!(GateError::CouldNotJudge("no runner".into()).exit_code(), 1);
}

#[test]
fn could_not_judge_is_not_a_verdict() {
    // Structural, not conventional: a gate that could not run cannot produce a
    // passing verdict because the type does not permit it.
    let outcome: Result<GateVerdict, GateError> = Err(GateError::CouldNotJudge("x".into()));
    assert!(outcome.is_err());
    assert_ne!(GateError::CouldNotJudge("x".into()).exit_code(), 0);
}

#[test]
fn a_failed_verdict_names_its_findings() {
    let verdict = GateVerdict::Failed {
        findings: vec!["a".into(), "b".into()],
    };
    assert_eq!(verdict.findings().len(), 2);
}
