//! Apple Silicon runner: the authoritative budget gate.
//!
//! It reproduces the baseline's performance and efficiency split, which is the
//! property that matters most. It does not reproduce the exact core count, so
//! the observed topology is recorded with every measurement and a baseline from
//! a different topology is rejected rather than compared.

use vulcan_app::ports::constrained_runner::{
    ConstrainedRunnerPort, ConstraintError, CoreTopology, MeasurementError,
};
use vulcan_domain::budget::{BudgetMeasurement, Runner};

const RUNNER: Runner = Runner::AppleSilicon;

pub struct AppleSiliconRunner;

impl ConstrainedRunnerPort for AppleSiliconRunner {
    fn assert_constraints(&self) -> Result<CoreTopology, ConstraintError> {
        if !cfg!(target_os = "macos") {
            return Err(ConstraintError::NotEnforceable(
                "not running on macOS; Apple Silicon topology cannot be observed".into(),
            ));
        }
        let performance = sysctl("hw.perflevel0.logicalcpu")?;
        let efficiency = sysctl("hw.perflevel1.logicalcpu").unwrap_or(0);
        if performance == 0 {
            return Err(ConstraintError::NotEnforceable(
                "no performance cores reported; this is not Apple Silicon".into(),
            ));
        }
        Ok(CoreTopology { performance, efficiency })
    }

    fn run_instrumented(&self, round_trip_ms: u32) -> Result<Vec<BudgetMeasurement>, MeasurementError> {
        // Enforcement is proven before anything is measured, so a report can
        // never describe an unconstrained machine.
        let topology = self
            .assert_constraints()
            .map_err(|ConstraintError::NotEnforceable(reason)| {
                MeasurementError::ProductFailedToStart(format!("constraints not enforced: {reason}"))
            })?;
        // Held for the whole run and removed when it drops, so an interrupted
        // measurement cannot leave a delay on the device for the next one.
        let _latency = super::latency::InjectedLatency::apply(round_trip_ms)?;
        super::run_measured(RUNNER, &topology.to_string(), round_trip_ms)
    }
}

fn sysctl(key: &str) -> Result<u8, ConstraintError> {
    let output = std::process::Command::new("sysctl")
        .args(["-n", key])
        .output()
        .map_err(|error| ConstraintError::NotEnforceable(format!("sysctl {key}: {error}")))?;
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|_| ConstraintError::NotEnforceable(format!("sysctl {key} returned no number")))
}
