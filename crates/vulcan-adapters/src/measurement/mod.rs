pub mod apple_silicon;
pub mod baseline;
pub mod instrument;
pub mod linux_cgroup;
pub mod netem;
pub mod report_store;

use vulcan_app::ports::constrained_runner::MeasurementError;
use vulcan_domain::budget::{BudgetMeasurement, Metric, Runner};

/// Spawns the product with instrumentation enabled and collects the report it
/// writes. The runner never measures: the process measures itself and this
/// reads the result, which is why the figures are exact rather than sampled.
pub(crate) fn run_measured(
    runner: Runner,
    core_topology: &str,
    round_trip_ms: u32,
) -> Result<Vec<BudgetMeasurement>, MeasurementError> {
    let report_path = std::env::temp_dir().join(format!("vulcan-measure-{round_trip_ms}.json"));
    let _ = std::fs::remove_file(&report_path);

    let status = std::process::Command::new("target/debug/shell-preview")
        .args(["--measure", report_path.to_str().unwrap_or_default()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|error| MeasurementError::ProductFailedToStart(error.to_string()))?;

    if !status.success() {
        return Err(MeasurementError::ProductFailedToStart(format!(
            "shell-preview exited with {status}"
        )));
    }

    let text = std::fs::read_to_string(&report_path)
        .map_err(|_| MeasurementError::MetricUnavailable("no report was written".into()))?;

    let parsed = crate::measurement::instrument::parse_report(&text);
    let measurements: Vec<BudgetMeasurement> = Metric::ALL
        .iter()
        .filter_map(|metric| {
            parsed
                .iter()
                .find(|(name, _)| name == metric.name())
                .map(|(_, measured)| BudgetMeasurement {
                    metric: *metric,
                    measured: *measured,
                    round_trip_ms,
                    runner,
                    core_topology: core_topology.to_string(),
                })
        })
        .collect();

    if measurements.is_empty() {
        return Err(MeasurementError::MetricUnavailable(
            "the report contained no metrics".into(),
        ));
    }
    Ok(measurements)
}
