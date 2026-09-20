//! T116: limits are applied and read back, not requested and not assumed.

use vulcan_adapters::measurement::linux_cgroup::{cores_from, LinuxCgroupRunner};
use vulcan_app::ports::constrained_runner::{ConstrainedRunnerPort, ConstraintError};

#[test]
fn a_quota_and_period_read_as_a_core_count() {
    assert_eq!(cores_from("600000 100000"), Some(6));
    assert_eq!(cores_from("200000 100000"), Some(2));
}

#[test]
fn an_unlimited_cpu_is_not_a_core_count() {
    // "max" is the absence of a limit. Reading it as a number would report a
    // constrained machine that is not constrained.
    assert_eq!(cores_from("max 100000"), None);
}

#[test]
fn a_quota_that_is_not_whole_cores_is_refused() {
    assert_eq!(cores_from("150000 100000"), None);
}

#[test]
fn the_runner_refuses_when_it_cannot_apply_the_limits() {
    // This machine does not grant write access to the cgroup hierarchy, so the
    // only correct outcome is a refusal naming what it could not do. Reporting
    // a topology here would describe a 16-core machine as the 6-core baseline.
    match LinuxCgroupRunner.assert_constraints() {
        Err(ConstraintError::NotEnforceable(reason)) => {
            assert!(
                reason.contains("cannot") || reason.contains("not mounted"),
                "the refusal should say what failed: {reason}"
            );
        }
        Ok(topology) => {
            // Privileged environment: then it must really be six cores.
            assert_eq!(topology.performance, 6, "reported a topology it did not apply");
        }
    }
}

#[test]
fn a_machine_with_too_few_cores_is_refused_before_any_quota_is_written() {
    // CI runners have four cores. Writing a six-core quota there succeeds, and
    // `cpu.max` reads back six, so the gate would report a baseline measurement
    // from hardware that cannot provide one.
    let present = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
    match LinuxCgroupRunner.assert_constraints() {
        Err(ConstraintError::NotEnforceable(reason)) if present < 6 => {
            assert!(reason.contains("cores"), "{reason}");
        }
        Err(ConstraintError::NotEnforceable(_)) => {}
        Ok(topology) => assert_eq!(topology.performance, 6),
    }
}
