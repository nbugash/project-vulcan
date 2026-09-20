//! Gate 5: measure every budget on the constrained machine.
//!
//! The rule that shapes this use case: refusing to measure and failing a
//! measurement are different outcomes. An environment that cannot enforce its
//! constraints yields no measurements at all, because a number from the wrong
//! machine is worse than no number.

use std::collections::HashMap;

use vulcan_domain::budget::{BudgetMeasurement, Metric, Verdict};
use vulcan_domain::verdict::{GateError, GateVerdict};

use crate::ports::constrained_runner::{ConstrainedRunnerPort, ConstraintError, MeasurementError};

pub struct MeasureBudgetsInput {
    pub round_trip_profiles: Vec<u32>,
    /// Previously accepted measurements, keyed by metric name. A baseline from a
    /// different core topology is rejected by the caller, not merged here.
    pub baseline: HashMap<String, f64>,
}

impl Default for MeasureBudgetsInput {
    fn default() -> Self {
        Self { round_trip_profiles: vec![0, 10, 30, 80], baseline: HashMap::new() }
    }
}

pub struct MeasureBudgetsOutput {
    pub verdict: GateVerdict,
    pub measurements: Vec<(BudgetMeasurement, Verdict)>,
    pub core_topology: String,
}

pub struct MeasureBudgets<R: ConstrainedRunnerPort> {
    runner: R,
}

impl<R: ConstrainedRunnerPort> MeasureBudgets<R> {
    pub fn new(runner: R) -> Self {
        Self { runner }
    }

    pub fn execute(&self, input: MeasureBudgetsInput) -> Result<MeasureBudgetsOutput, GateError> {
        let topology = self
            .runner
            .assert_constraints()
            .map_err(|ConstraintError::NotEnforceable(reason)| {
                GateError::CouldNotJudge(format!(
                    "constraints not enforced: {reason}; refusing to report unconstrained measurements"
                ))
            })?;

        let mut judged = Vec::new();
        for profile in &input.round_trip_profiles {
            let measurements = self.runner.run_instrumented(*profile).map_err(|error| {
                GateError::CouldNotJudge(match error {
                    MeasurementError::ProductFailedToStart(detail) => {
                        format!("product failed to start: {detail}")
                    }
                    MeasurementError::MetricUnavailable(metric) => {
                        format!("metric {metric} could not be measured; a partial report is not a report")
                    }
                })
            })?;

            // A run that omits a metric it could have produced fails rather than
            // reporting a partial set. A metric nothing in the product can
            // exercise yet is not omitted, it is not yet applicable.
            for metric in Metric::ALL {
                if !metric.measurable_by_the_shell() {
                    continue;
                }
                let expected = !metric.round_trip_sensitive() && *profile != 0;
                if !expected && !measurements.iter().any(|m| m.metric == metric) {
                    return Err(GateError::CouldNotJudge(format!(
                        "metric {} missing from the run at {}ms round trip",
                        metric.name(),
                        profile
                    )));
                }
            }

            for measurement in measurements {
                let baseline = input.baseline.get(measurement.metric.name()).copied();
                let verdict = measurement.judge(baseline);
                judged.push((measurement, verdict));
            }
        }

        let findings: Vec<String> = judged
            .iter()
            .filter(|(measurement, verdict)| {
                *verdict != Verdict::Pass && measurement.runner.is_authoritative()
            })
            .map(|(measurement, verdict)| {
                let label = if *verdict == Verdict::Fail { "over budget" } else { "regressed" };
                format!(
                    "{} {}: measured {:.1}{}, budget {:.1}{} at {}ms round trip",
                    measurement.metric.name(),
                    label,
                    measurement.measured,
                    measurement.metric.unit(),
                    measurement.metric.budget(),
                    measurement.metric.unit(),
                    measurement.round_trip_ms
                )
            })
            .collect();

        let blocking = judged
            .iter()
            .any(|(m, v)| *v == Verdict::Fail && m.runner.is_authoritative());

        let verdict = match (blocking, findings.is_empty()) {
            (true, _) => GateVerdict::Failed { findings },
            (false, false) => GateVerdict::Regressed { findings },
            (false, true) => GateVerdict::Passed,
        };

        Ok(MeasureBudgetsOutput { verdict, measurements: judged, core_topology: topology.to_string() })
    }
}
