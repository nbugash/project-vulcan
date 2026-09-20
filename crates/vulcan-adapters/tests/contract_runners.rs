//! T035: the constrained runner contract, against the real adapters.
//!
//! The same suite the fake passes, run against `LinuxCgroupRunner` and
//! `AppleSiliconRunner`. Without this the fake is free to drift into a shape no
//! real adapter has, and every use-case test built on it means nothing.

mod contract {
    include!("../../vulcan-app/tests/contract/mod.rs");
}

use vulcan_adapters::measurement::apple_silicon::AppleSiliconRunner;
use vulcan_adapters::measurement::linux_cgroup::LinuxCgroupRunner;
use vulcan_app::ports::constrained_runner::ConstrainedRunnerPort;

#[test]
fn linux_cgroup_runner_satisfies_the_contract() {
    contract::constrained_runner_contract(&LinuxCgroupRunner);
    contract::constrained_runner_refuses_to_measure_unconstrained(&LinuxCgroupRunner);
}

#[test]
fn apple_silicon_runner_satisfies_the_contract() {
    contract::constrained_runner_contract(&AppleSiliconRunner);
    contract::constrained_runner_refuses_to_measure_unconstrained(&AppleSiliconRunner);
}

#[test]
fn a_runner_off_its_platform_refuses_rather_than_guessing() {
    // Exactly one of these can enforce on any given machine, and the other must
    // say why rather than inventing a topology.
    let linux = LinuxCgroupRunner.assert_constraints();
    let apple = AppleSiliconRunner.assert_constraints();
    assert!(
        linux.is_err() || apple.is_err(),
        "both runners claimed enforcement on the same machine, which no machine satisfies"
    );
    for outcome in [linux, apple] {
        if let Err(vulcan_app::ports::constrained_runner::ConstraintError::NotEnforceable(reason)) = outcome {
            assert!(reason.len() > 10, "a refusal must be specific enough to act on: {reason}");
        }
    }
}
