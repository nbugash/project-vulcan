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
    // A cold start is only cold if the product's own files are.
    //
    // Left alone, the first launch finds the binary resident from the build that
    // produced it and reports a warm figure under a cold name — every cold start
    // this project recorded before this was a second launch in disguise.
    //
    // Evicting the whole page cache is the other extreme and is not a cold
    // start either: nobody launches an editor on a machine where libc and mesa
    // are unread. What a first launch of the day actually costs is our own
    // binary off disk against a system already warm, which is what this evicts.
    // It needs no privilege, so the same measurement is available in CI.
    let binary = preview_binary();
    evict_from_page_cache(&binary);

    let report_path = std::env::temp_dir().join(format!("vulcan-measure-{round_trip_ms}.json"));
    let _ = std::fs::remove_file(&report_path);

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

    // A second launch, measured the same way. The first paid for reading the
    // binary and its libraries off disk; this one finds them in the page cache,
    // and the difference between the two is what that caching is worth.
    let warm_path = std::env::temp_dir().join(format!("vulcan-measure-warm-{round_trip_ms}.json"));
    let _ = std::fs::remove_file(&warm_path);
    let warm = std::process::Command::new(&binary)
        .args(["--measure", warm_path.to_str().unwrap_or_default()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()
        .filter(|status| status.success())
        .and_then(|_| std::fs::read_to_string(&warm_path).ok())
        .and_then(|text| {
            crate::measurement::instrument::parse_report(&text)
                .into_iter()
                .find(|(name, _)| name == Metric::ColdStart.name())
                .map(|(_, measured)| measured)
        });

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
    let mut measurements: Vec<BudgetMeasurement> = Metric::ALL
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
    if let Some(measured) = warm {
        measurements.push(BudgetMeasurement {
            metric: Metric::WarmStart,
            measured,
            round_trip_ms,
            runner,
            core_topology: core_topology.to_string(),
        });
    }

    Ok(measurements)
}
pub mod latency;

/// The shell binary the budgets are measured against.
///
/// Release when it exists, because that is what ships and what the budgets
/// describe. The debug binary is 508 MB against release's 25, and reading the
/// difference off a cold disk is over a second of cold start — a figure about
/// the build profile rather than the product.
fn preview_binary() -> std::path::PathBuf {
    let target = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let release = std::path::Path::new(&target).join("release").join("shell-preview");
    if release.exists() {
        return release;
    }
    std::path::Path::new(&target).join("debug").join("shell-preview")
}

/// Asks the kernel to drop this file from the page cache.
///
/// `POSIX_FADV_DONTNEED` over the whole file. Advice rather than a command: the
/// kernel keeps pages another process is using, which is the behaviour wanted
/// here — system libraries stay resident and only the product goes cold.
fn evict_from_page_cache(path: &std::path::Path) {
    use std::os::unix::io::AsRawFd;

    let Ok(file) = std::fs::File::open(path) else {
        return;
    };
    let length = file.metadata().map(|m| m.len()).unwrap_or(0) as libc::off_t;

    // Safety: a file this process opened, advised over its own length.
    unsafe {
        libc::posix_fadvise(file.as_raw_fd(), 0, length, libc::POSIX_FADV_DONTNEED);
    }
}
