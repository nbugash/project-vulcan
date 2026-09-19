//! T033: the budget verdict rule.

use vulcan_domain::budget::{BudgetMeasurement, Metric, Runner, Verdict};

fn measurement(metric: Metric, measured: f64) -> BudgetMeasurement {
    BudgetMeasurement {
        metric,
        measured,
        round_trip_ms: 0,
        runner: Runner::AppleSilicon,
        core_topology: "2P+4E".into(),
    }
}

#[test]
fn under_budget_with_no_baseline_passes() {
    assert_eq!(measurement(Metric::ColdStart, 250.0).judge(None), Verdict::Pass);
}

#[test]
fn over_budget_fails() {
    assert_eq!(measurement(Metric::ColdStart, 301.0).judge(None), Verdict::Fail);
}

#[test]
fn exactly_at_budget_passes() {
    assert_eq!(measurement(Metric::ColdStart, 300.0).judge(None), Verdict::Pass);
}

#[test]
fn ten_percent_worse_than_baseline_regresses_while_under_budget() {
    // 200 -> 220.1 is worse than ten percent, and still inside the 300ms budget.
    assert_eq!(measurement(Metric::ColdStart, 220.1).judge(Some(200.0)), Verdict::Regressed);
}

#[test]
fn exactly_ten_percent_worse_is_not_yet_a_regression() {
    assert_eq!(measurement(Metric::ColdStart, 220.0).judge(Some(200.0)), Verdict::Pass);
}

#[test]
fn over_budget_beats_regression_when_both_apply() {
    assert_eq!(measurement(Metric::ColdStart, 400.0).judge(Some(200.0)), Verdict::Fail);
}

#[test]
fn the_constitution_defines_thirteen_budgets() {
    assert_eq!(Metric::ALL.len(), 13);
    for metric in Metric::ALL {
        assert!(metric.budget() > 0.0, "{} has no budget", metric.name());
    }
}

#[test]
fn frame_budgets_are_one_frame_at_120hz() {
    assert_eq!(Metric::KeystrokeToPaint.budget(), 8.0);
    assert_eq!(Metric::ScrollTickToPaint.budget(), 8.0);
    assert_eq!(Metric::LongestUiThreadTask.budget(), 8.0);
}

#[test]
fn only_apple_silicon_is_authoritative() {
    assert!(Runner::AppleSilicon.is_authoritative());
    assert!(!Runner::LinuxCgroup.is_authoritative());
}
