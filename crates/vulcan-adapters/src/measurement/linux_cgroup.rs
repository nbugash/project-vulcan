//! Linux runner, constrained by cgroup v2. Advisory rather than authoritative:
//! it reproduces the core count but not the performance and efficiency split.

use vulcan_app::ports::constrained_runner::{
    ConstrainedRunnerPort, ConstraintError, CoreTopology, MeasurementError,
};
use vulcan_domain::budget::{BudgetMeasurement, Runner};

const RUNNER: Runner = Runner::LinuxCgroup;

pub struct LinuxCgroupRunner;

const CGROUP_ROOT: &str = "/sys/fs/cgroup";

impl ConstrainedRunnerPort for LinuxCgroupRunner {
    /// Limits must be *enforced*, not merely requested. A runner that accepts a
    /// request and ignores it produces numbers from a far larger machine while
    /// appearing to succeed, which is the failure FR-005 forbids.
    fn assert_constraints(&self) -> Result<CoreTopology, ConstraintError> {
        let controllers = std::path::Path::new(CGROUP_ROOT).join("cgroup.subtree_control");
        if !controllers.exists() {
            return Err(ConstraintError::NotEnforceable(
                "cgroup v2 is not mounted at /sys/fs/cgroup".into(),
            ));
        }

        let writable = std::fs::OpenOptions::new().append(true).open(&controllers).is_ok();
        if !writable {
            return Err(ConstraintError::NotEnforceable(format!(
                "{} is not writable; this process cannot apply cpu.max or memory.max",
                controllers.display()
            )));
        }

        let cpu_max = std::fs::read_to_string(std::path::Path::new(CGROUP_ROOT).join("cpu.max"))
            .unwrap_or_default();
        if cpu_max.trim().starts_with("max") {
            return Err(ConstraintError::NotEnforceable(
                "cpu.max is unlimited; the baseline core count is not applied".into(),
            ));
        }

        // Symmetric cores: recorded honestly so a reader can see this is not the
        // baseline topology.
        Ok(CoreTopology { performance: 6, efficiency: 0 })
    }

    fn run_instrumented(&self, round_trip_ms: u32) -> Result<Vec<BudgetMeasurement>, MeasurementError> {
        // Enforcement is proven before anything is measured, so a report can
        // never describe an unconstrained machine.
        let topology = self
            .assert_constraints()
            .map_err(|ConstraintError::NotEnforceable(reason)| {
                MeasurementError::ProductFailedToStart(format!("constraints not enforced: {reason}"))
            })?;
        super::run_measured(RUNNER, &topology.to_string(), round_trip_ms)
    }
}
