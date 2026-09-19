//! Linux runner, constrained by cgroup v2. Advisory rather than authoritative:
//! it reproduces the core count but not the performance and efficiency split.

use vulcan_app::ports::constrained_runner::{
    ConstrainedRunnerPort, ConstraintError, CoreTopology, MeasurementError,
};
use vulcan_domain::budget::{BudgetMeasurement, Runner};

const RUNNER: Runner = Runner::LinuxCgroup;

pub struct LinuxCgroupRunner;

const CGROUP_ROOT: &str = "/sys/fs/cgroup";
const GROUP: &str = "vulcan-budget";
const PERIOD_US: u64 = 100_000;
/// The baseline machine: 6 cores and 8 GB.
const CORES: u32 = 6;
const MEMORY_BYTES: u64 = 8 * 1024 * 1024 * 1024;

impl ConstrainedRunnerPort for LinuxCgroupRunner {
    /// Limits must be *applied* and then read back, not requested and not
    /// assumed. An earlier version only checked that something else had already
    /// set `cpu.max`, and reported a hardcoded six-core topology whatever the
    /// machine was — which is a fabricated measurement context, the exact thing
    /// FR-005 forbids.
    fn assert_constraints(&self) -> Result<CoreTopology, ConstraintError> {
        let root = std::path::Path::new(CGROUP_ROOT);
        if !root.join("cgroup.controllers").exists() {
            return Err(ConstraintError::NotEnforceable(
                "cgroup v2 is not mounted at /sys/fs/cgroup".into(),
            ));
        }

        let group = root.join(GROUP);
        std::fs::create_dir_all(&group).map_err(|error| {
            ConstraintError::NotEnforceable(format!(
                "cannot create {}: {error}; applying limits needs write access to the \
                 cgroup hierarchy, usually root or a delegated subtree",
                group.display()
            ))
        })?;

        // Delegation first: a controller that is not enabled in the parent
        // cannot be set in the child, and the write below would be ignored.
        write(&root.join("cgroup.subtree_control"), "+cpu +memory")?;

        // 6 cores: 600000us of runtime per 100000us period.
        write(&group.join("cpu.max"), &format!("{} {PERIOD_US}", CORES as u64 * PERIOD_US))?;
        write(&group.join("memory.max"), &MEMORY_BYTES.to_string())?;
        // Nothing is constrained until this process is actually inside it.
        write(&group.join("cgroup.procs"), &std::process::id().to_string())?;

        verify(&group)
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

fn write(path: &std::path::Path, value: &str) -> Result<(), ConstraintError> {
    std::fs::write(path, value).map_err(|error| {
        ConstraintError::NotEnforceable(format!("cannot write {}: {error}", path.display()))
    })
}

/// Reads the limits back out of the kernel. What was asked for is not evidence;
/// what the kernel reports is.
fn verify(group: &std::path::Path) -> Result<CoreTopology, ConstraintError> {
    let cpu = std::fs::read_to_string(group.join("cpu.max")).map_err(|error| {
        ConstraintError::NotEnforceable(format!("cannot read cpu.max: {error}"))
    })?;
    let cores = cores_from(&cpu).ok_or_else(|| {
        ConstraintError::NotEnforceable(format!("cpu.max reads {:?}, which is not a limit", cpu.trim()))
    })?;
    if cores != CORES {
        return Err(ConstraintError::NotEnforceable(format!(
            "asked for {CORES} cores, the kernel reports {cores}"
        )));
    }

    let memory = std::fs::read_to_string(group.join("memory.max")).map_err(|error| {
        ConstraintError::NotEnforceable(format!("cannot read memory.max: {error}"))
    })?;
    let applied: u64 = memory.trim().parse().map_err(|_| {
        ConstraintError::NotEnforceable(format!("memory.max reads {:?}, which is not a limit", memory.trim()))
    })?;
    if applied != MEMORY_BYTES {
        return Err(ConstraintError::NotEnforceable(format!(
            "asked for {MEMORY_BYTES} bytes, the kernel reports {applied}"
        )));
    }

    // Symmetric, and said so: this is not the baseline's two performance plus
    // four efficiency cores, and a reader of the report must be able to see that.
    Ok(CoreTopology { performance: cores as u8, efficiency: 0 })
}

/// `"600000 100000"` -> 6 cores. `"max 100000"` is no limit at all.
pub fn cores_from(cpu_max: &str) -> Option<u32> {
    let mut parts = cpu_max.split_whitespace();
    let quota: u64 = parts.next()?.parse().ok()?;
    let period: u64 = parts.next()?.parse().ok()?;
    (period > 0 && quota % period == 0).then(|| (quota / period) as u32)
}
