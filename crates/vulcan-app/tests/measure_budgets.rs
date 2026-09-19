//! T034, T036: the budget use case against a fake runner.

use std::collections::HashMap;

use vulcan_app::ports::constrained_runner::{
    ConstrainedRunnerPort, ConstraintError, CoreTopology, MeasurementError,
};
use vulcan_app::use_cases::measure_budgets::{MeasureBudgets, MeasureBudgetsInput};
use vulcan_domain::budget::{BudgetMeasurement, Metric, Runner};
use vulcan_domain::verdict::GateError;

struct FakeRunner {
    topology: Result<CoreTopology, ConstraintError>,
    measurements: Result<Vec<BudgetMeasurement>, MeasurementError>,
}

impl FakeRunner {
    fn healthy(value_for_cold_start: f64) -> Self {
        let measurements = Metric::ALL
            .iter()
            .map(|metric| BudgetMeasurement {
                metric: *metric,
                measured: if *metric == Metric::ColdStart { value_for_cold_start } else { 1.0 },
                round_trip_ms: 0,
                runner: Runner::AppleSilicon,
                core_topology: "2P+4E".into(),
            })
            .collect();
        Self {
            topology: Ok(CoreTopology { performance: 2, efficiency: 4 }),
            measurements: Ok(measurements),
        }
    }
}

impl ConstrainedRunnerPort for FakeRunner {
    fn assert_constraints(&self) -> Result<CoreTopology, ConstraintError> {
        self.topology.clone()
    }
    fn run_instrumented(&self, _: u32) -> Result<Vec<BudgetMeasurement>, MeasurementError> {
        self.measurements.clone()
    }
}

fn input() -> MeasureBudgetsInput {
    MeasureBudgetsInput { round_trip_profiles: vec![0], baseline: HashMap::new() }
}

#[test]
fn a_healthy_run_passes_and_reports_every_metric() {
    let output = MeasureBudgets::new(FakeRunner::healthy(250.0)).execute(input()).unwrap();
    assert_eq!(output.measurements.len(), 13);
    assert_eq!(output.core_topology, "2P+4E");
    assert!(!output.verdict.is_blocking());
}

#[test]
fn an_over_budget_metric_fails_and_names_it() {
    let output = MeasureBudgets::new(FakeRunner::healthy(999.0)).execute(input()).unwrap();
    assert!(output.verdict.is_blocking());
    let finding = &output.verdict.findings()[0];
    assert!(finding.contains("ColdStart"), "{finding}");
    assert!(finding.contains("999"), "{finding}");
    assert!(finding.contains("300"), "{finding}");
}

#[test]
fn unenforceable_constraints_produce_could_not_judge_and_no_measurements() {
    let runner = FakeRunner {
        topology: Err(ConstraintError::NotEnforceable("cgroup not writable".into())),
        measurements: Ok(Vec::new()),
    };
    match MeasureBudgets::new(runner).execute(input()) {
        Err(GateError::CouldNotJudge(reason)) => {
            assert!(reason.contains("cgroup not writable"), "{reason}");
            assert!(reason.contains("refusing"), "{reason}");
        }
        other => panic!("expected CouldNotJudge, got {:?}", other.map(|o| o.verdict)),
    }
}

#[test]
fn a_run_missing_a_metric_refuses_rather_than_reporting_a_partial_set() {
    let mut runner = FakeRunner::healthy(1.0);
    if let Ok(measurements) = &mut runner.measurements {
        measurements.retain(|m| m.metric != Metric::IdleCpu);
    }
    match MeasureBudgets::new(runner).execute(input()) {
        Err(GateError::CouldNotJudge(reason)) => assert!(reason.contains("IdleCpu"), "{reason}"),
        other => panic!("expected CouldNotJudge, got {:?}", other.map(|o| o.verdict)),
    }
}

#[test]
fn a_regression_within_budget_reports_without_blocking() {
    let mut baseline = HashMap::new();
    baseline.insert("ColdStart".to_string(), 200.0);
    let output = MeasureBudgets::new(FakeRunner::healthy(250.0))
        .execute(MeasureBudgetsInput { round_trip_profiles: vec![0], baseline })
        .unwrap();
    assert!(!output.verdict.is_blocking());
    assert!(output.verdict.findings().iter().any(|f| f.contains("regressed")));
}
