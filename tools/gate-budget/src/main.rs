//! Gate 5: resource budgets.
//!
//! Exit 1 when the runner cannot enforce its constraints: refusing to measure is
//! a distinct outcome from failing a measurement, and neither is a pass.

use std::collections::HashMap;
use std::path::PathBuf;

use vulcan_adapters::measurement::apple_silicon::AppleSiliconRunner;
use vulcan_adapters::measurement::baseline::Baseline;
use vulcan_adapters::measurement::linux_cgroup::LinuxCgroupRunner;
use vulcan_adapters::measurement::latency::PROFILES;
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

    // `--report` is consumed into `args.report_path` by the shared parser and
    // never reaches `flags`, so the `args.flag("report")` this used to ask for
    // was always None and the measurement report was never written. Every
    // budget report this gate has produced held a verdict and no numbers.
    //
    // Beside the gate report rather than onto it: the verdict summary has the
    // same shape for every gate and CI reads it, so it keeps the requested path
    // and the detail moves one name over.
    if let (Some(report), Some(path)) = (report, args.report_path.as_deref()) {
        let path = measurements_path(std::path::Path::new(path));
        if let Err(error) = report.write(&path) {
            eprintln!("warning: could not write measurements to {}: {error}", path.display());
        }
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

/// `reports/budgets/linux-cgroup.json` becomes
/// `reports/budgets/linux-cgroup-measurements.json`.
fn measurements_path(report: &std::path::Path) -> PathBuf {
    let stem = report.file_stem().and_then(|s| s.to_str()).unwrap_or("budgets");
    let extension = report.extension().and_then(|s| s.to_str()).unwrap_or("json");
    report.with_file_name(format!("{stem}-measurements.{extension}"))
}

#[cfg(test)]
mod tests {
    use super::measurements_path;
    use std::path::Path;

    #[test]
    fn the_detail_lands_beside_the_summary_and_never_on_it() {
        let summary = Path::new("reports/budgets/linux-cgroup.json");
        let detail = measurements_path(summary);
        assert_eq!(detail, Path::new("reports/budgets/linux-cgroup-measurements.json"));
        assert_ne!(detail, summary);
    }
}
