//! Gate 5: resource budgets.
//!
//! Exit 1 when the runner cannot enforce its constraints: refusing to measure is
//! a distinct outcome from failing a measurement, and neither is a pass.

use std::collections::HashMap;
use std::path::PathBuf;

use vulcan_adapters::measurement::apple_silicon::AppleSiliconRunner;
use vulcan_adapters::measurement::baseline::Baseline;
use vulcan_adapters::measurement::linux_cgroup::LinuxCgroupRunner;
use vulcan_adapters::measurement::netem::PROFILES;
use vulcan_adapters::measurement::report_store::MeasurementReport;
use vulcan_app::ports::constrained_runner::ConstrainedRunnerPort;
use vulcan_app::use_cases::measure_budgets::{MeasureBudgets, MeasureBudgetsInput};
use vulcan_cli::gate_entry::{finish, Args};
use vulcan_domain::verdict::{GateError, GateVerdict};

fn main() {
    let args = Args::parse(std::env::args().skip(1));
    let runner_name = args.flag("runner").unwrap_or("linux-cgroup").to_string();

    let profiles: Vec<u32> = match args.flag("rtt") {
        Some(value) if !value.is_empty() => {
            value.split(',').filter_map(|item| item.trim().parse().ok()).collect()
        }
        _ => PROFILES.to_vec(),
    };

    let (outcome, report) = match runner_name.as_str() {
        "apple-silicon" => execute(AppleSiliconRunner, &runner_name, true, profiles, &args),
        "linux-cgroup" => execute(LinuxCgroupRunner, &runner_name, false, profiles, &args),
        other => (
            Err(GateError::CouldNotJudge(format!("unknown runner {other}"))),
            None,
        ),
    };

    if let (Some(report), Some(path)) = (report, args.flag("report").map(PathBuf::from)) {
        let _ = report.write(&path);
    }

    let context = vec![
        ("runner".into(), runner_name.clone()),
        ("authoritative".into(), (runner_name == "apple-silicon").to_string()),
    ];
    std::process::exit(finish("gate-budget", &args, outcome, context));
}

fn execute<R: ConstrainedRunnerPort>(
    runner: R,
    name: &str,
    authoritative: bool,
    profiles: Vec<u32>,
    args: &Args,
) -> (Result<GateVerdict, GateError>, Option<MeasurementReport>) {
    let use_case = MeasureBudgets::new(runner);

    let baseline = args
        .flag("baseline")
        .map(PathBuf::from)
        .and_then(|path| Baseline::load(&path, "").ok())
        .map(|loaded| loaded.values)
        .unwrap_or_else(HashMap::new);

    match use_case.execute(MeasureBudgetsInput { round_trip_profiles: profiles, baseline }) {
        Ok(output) => {
            let report = MeasurementReport {
                runner: name.to_string(),
                authoritative,
                core_topology: output.core_topology.clone(),
                measurements: output.measurements,
            };
            (Ok(output.verdict), Some(report))
        }
        Err(error) => (Err(error), None),
    }
}
