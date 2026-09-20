pub mod apple_silicon;
pub mod baseline;
pub mod instrument;
pub mod linux_cgroup;
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

    let target = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let binary = std::path::Path::new(&target).join("debug").join("shell-preview");
    let status = std::process::Command::new(&binary)
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

    let text = std::fs::read_to_string(&report_path).map_err(|_| {
        MeasurementError::MetricUnavailable(format!(
            "no report was written to {}",
            report_path.display()
        ))
    })?;

    let parsed = crate::measurement::instrument::parse_report(&text);

    // When a metric is missing, the useful question is what the run did
    // produce. Saying so turns "KeystrokeToPaint missing" from a dead end into
    // something a reader can act on, and the report stays on disk to inspect.
    if !parsed.iter().any(|(name, _)| name == Metric::KeystrokeToPaint.name()) {
        let produced: Vec<&str> = parsed.iter().map(|(name, _)| name.as_str()).collect();
        eprintln!(
            "note: the measured run produced {:?} and is kept at {}",
            produced,
            report_path.display()
        );
    }
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
pub mod latency;
