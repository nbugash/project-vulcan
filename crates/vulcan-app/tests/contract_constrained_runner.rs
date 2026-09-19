//! T035: the constrained runner contract, against the in-memory fake.
//!
//! The same suite runs against the real adapters in `vulcan-adapters`, which is
//! what keeps the fake honest.

mod contract {
    include!("contract/mod.rs");
}

use vulcan_app::ports::constrained_runner::{
    ConstrainedRunnerPort, ConstraintError, CoreTopology, MeasurementError,
};
use vulcan_domain::budget::{BudgetMeasurement, Metric, Runner};

/// The fake used by use-case tests: enforcement succeeds and every metric is
/// reported, which is the shape a real runner must also produce.
struct FakeRunner;

impl ConstrainedRunnerPort for FakeRunner {
    fn assert_constraints(&self) -> Result<CoreTopology, ConstraintError> {
        Ok(CoreTopology { performance: 2, efficiency: 4 })
    }

    fn run_instrumented(&self, round_trip_ms: u32) -> Result<Vec<BudgetMeasurement>, MeasurementError> {
        Ok(Metric::ALL
            .iter()
            .map(|metric| BudgetMeasurement {
                metric: *metric,
                measured: 1.0,
                round_trip_ms,
                runner: Runner::AppleSilicon,
                core_topology: "2P+4E".into(),
            })
            .collect())
    }
}

/// A runner that cannot enforce, to prove the refusal half of the contract.
struct UnenforceableRunner;

impl ConstrainedRunnerPort for UnenforceableRunner {
    fn assert_constraints(&self) -> Result<CoreTopology, ConstraintError> {
        Err(ConstraintError::NotEnforceable("no cgroup controller".into()))
    }
    fn run_instrumented(&self, _: u32) -> Result<Vec<BudgetMeasurement>, MeasurementError> {
        Err(MeasurementError::ProductFailedToStart("constraints not enforced".into()))
    }
}

#[test]
fn fake_satisfies_the_contract() {
    contract::constrained_runner_contract(&FakeRunner);
    contract::constrained_runner_refuses_to_measure_unconstrained(&FakeRunner);
}

#[test]
fn an_unenforceable_runner_satisfies_the_contract_by_refusing() {
    contract::constrained_runner_contract(&UnenforceableRunner);
    contract::constrained_runner_refuses_to_measure_unconstrained(&UnenforceableRunner);
}

#[test]
fn a_runner_that_measures_without_enforcing_violates_the_contract() {
    // A deliberately wrong adapter: refuses enforcement, measures anyway. The
    // suite must catch it, or it catches nothing.
    struct Dishonest;
    impl ConstrainedRunnerPort for Dishonest {
        fn assert_constraints(&self) -> Result<CoreTopology, ConstraintError> {
            Err(ConstraintError::NotEnforceable("unconstrained".into()))
        }
        fn run_instrumented(&self, _: u32) -> Result<Vec<BudgetMeasurement>, MeasurementError> {
            Ok(Vec::new())
        }
    }
    let caught = std::panic::catch_unwind(|| {
        contract::constrained_runner_refuses_to_measure_unconstrained(&Dishonest)
    });
    assert!(caught.is_err(), "the contract must reject a runner that measures unconstrained");
}
