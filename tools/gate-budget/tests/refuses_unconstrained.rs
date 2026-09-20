//! T047: the gate refuses rather than reporting unconstrained measurements.
//!
//! This is the scenario that matters most in gate 5. A measurement taken on an
//! unconstrained machine is not a weaker measurement, it is a measurement of a
//! different machine, and reporting it would be worse than reporting nothing.

use std::process::Command;

fn run(args: &[&str]) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_gate-budget"))
        .args(args)
        .output()
        .expect("gate-budget runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
    )
}

#[test]
fn an_unenforceable_runner_exits_one_and_writes_nothing() {
    let report = std::env::temp_dir().join("vulcan-budget-should-not-exist.json");
    let _ = std::fs::remove_file(&report);

    let (code, output) = run(&["--runner", "linux-cgroup", "--report", report.to_str().unwrap()]);

    // Exit 1, not 2: could-not-judge is not a failing measurement.
    assert_eq!(code, 1, "{output}");
    assert!(output.contains("COULD NOT JUDGE"), "{output}");
    assert!(
        !report.exists(),
        "no measurement file may be written when constraints were not enforced"
    );
}

#[test]
fn the_refusal_names_the_constraint_that_could_not_be_applied() {
    let (_, output) = run(&["--runner", "linux-cgroup"]);
    assert!(output.contains("refusing to report unconstrained measurements"), "{output}");
}

#[test]
fn an_unknown_runner_cannot_be_judged() {
    let (code, output) = run(&["--runner", "guesswork"]);
    assert_eq!(code, 1, "{output}");
    assert!(output.contains("unknown runner guesswork"), "{output}");
}

#[test]
fn json_output_reports_the_runner_and_its_authority() {
    let (_, output) = run(&["--runner", "apple-silicon", "--json"]);
    assert!(output.contains("\"status\": \"could_not_judge\""), "{output}");
    assert!(output.contains("\"authoritative\": \"true\""), "{output}");
}
