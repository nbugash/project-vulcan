//! In-process instrumentation.
//!
//! Measured from inside the process rather than observed from outside. An
//! external watcher can only sample, and a sampler that runs at 100 Hz cannot
//! see an 8 ms budget: it reports a distribution of guesses where the frame loop
//! already knows the exact number. Every figure here is recorded by the code
//! that performs the work.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use vulcan_domain::budget::{BudgetMeasurement, Metric, Runner, Statistic};

/// What a recorded span was doing, so it lands against the right budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
    KeystrokeToPaint,
    ScrollTickToPaint,
    UiThreadTask,
    HighlightUpdate,
}

impl Span {
    fn metric(self) -> Metric {
        match self {
            Span::KeystrokeToPaint => Metric::KeystrokeToPaint,
            Span::ScrollTickToPaint => Metric::ScrollTickToPaint,
            Span::UiThreadTask => Metric::LongestUiThreadTask,
            Span::HighlightUpdate => Metric::HighlightUpdate,
        }
    }
}

/// Records spans and process figures for one run.
pub struct Instrument {
    started: Instant,
    first_frame: Mutex<Option<Duration>>,
    spans: Mutex<Vec<(Span, Duration)>>,
    idle_busy_nanos: AtomicU64,
    idle_window_nanos: AtomicU64,
    peak_memory_kb: AtomicU64,
}

impl Default for Instrument {
    fn default() -> Self {
        Self::new()
    }
}

impl Instrument {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            first_frame: Mutex::new(None),
            spans: Mutex::new(Vec::new()),
            idle_busy_nanos: AtomicU64::new(0),
            idle_window_nanos: AtomicU64::new(0),
            peak_memory_kb: AtomicU64::new(0),
        }
    }

    /// Cold start is measured to the first frame the user could see, not to the
    /// end of `main`. Indexing, plugins and language servers are excluded by
    /// construction: none of them has run yet.
    pub fn mark_first_frame(&self) {
        let mut slot = self.first_frame.lock().expect("instrument lock");
        if slot.is_none() {
            *slot = Some(self.started.elapsed());
        }
    }

    /// Times a span exactly. The closure's return value passes through, so
    /// instrumenting a call site does not change its shape.
    pub fn record<T>(&self, span: Span, work: impl FnOnce() -> T) -> T {
        let started = Instant::now();
        let value = work();
        self.observe(span, started.elapsed());
        value
    }

    pub fn observe(&self, span: Span, elapsed: Duration) {
        self.spans.lock().expect("instrument lock").push((span, elapsed));
        self.sample_memory();
    }

    /// Time spent working while the product was otherwise idle, against the
    /// wall-clock window that work sat in. Both halves of the ratio come from
    /// the caller: a reporter that assumed a window would silently rescale
    /// every figure when the sampling period changed.
    pub fn observe_idle(&self, busy: Duration, window: Duration) {
        self.idle_busy_nanos
            .fetch_add(busy.as_nanos() as u64, Ordering::Relaxed);
        self.idle_window_nanos
            .fetch_add(window.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Watches the idle window from inside, using processor time rather than
    /// timer drift. Returns once the window has elapsed.
    ///
    /// The caller used to compute `busy` by subtracting a requested sleep from
    /// its actual duration, which measures how coarse the platform's timers are.
    /// On macOS that reported ten percent of a core for a window in which the
    /// product did nothing.
    pub fn observe_idle_over(&self, window: Duration) {
        let before = cpu_time();
        let started = Instant::now();
        std::thread::sleep(window);
        let elapsed = started.elapsed();

        if let (Some(before), Some(after)) = (before, cpu_time()) {
            self.observe_idle(after.saturating_sub(before), elapsed);
        }
    }

    pub fn sample_memory(&self) {
        if let Some(kb) = resident_kb() {
            self.peak_memory_kb.fetch_max(kb, Ordering::Relaxed);
        }
    }

    /// Reduces a run of samples the way the budget is stated.
    ///
    /// Never the average: a budget describes the frame a user noticed, and the
    /// mean hides exactly that frame. But "p99" is not "the worst", either, and
    /// treating them as the same makes a verdict turn on one hiccup from some
    /// other process on the machine.
    fn reduce(&self, span: Span, statistic: Statistic) -> Option<Duration> {
        let mut samples: Vec<Duration> = self
            .spans
            .lock()
            .expect("instrument lock")
            .iter()
            .filter(|(kind, _)| *kind == span)
            .map(|(_, elapsed)| *elapsed)
            .collect();
        if samples.is_empty() {
            return None;
        }
        samples.sort_unstable();

        Some(match statistic {
            Statistic::Max => samples[samples.len() - 1],
            Statistic::Percentile(p) => {
                // Nearest-rank: the smallest sample at or above the pth
                // percentile, which for a small run is the last one rather than
                // an interpolation between samples that were never taken.
                let rank = ((p / 100.0) * samples.len() as f64).ceil() as usize;
                samples[rank.clamp(1, samples.len()) - 1]
            }
        })
    }

    /// How many samples a span has, so a report can say whether a percentile
    /// over them means anything.
    pub fn sample_count(&self, span: Span) -> usize {
        self.spans
            .lock()
            .expect("instrument lock")
            .iter()
            .filter(|(kind, _)| *kind == span)
            .count()
    }

    pub fn report(&self, runner: Runner, core_topology: &str, round_trip_ms: u32) -> Vec<BudgetMeasurement> {
        let measurement = |metric: Metric, measured: f64| BudgetMeasurement {
            metric,
            measured,
            round_trip_ms,
            runner,
            core_topology: core_topology.to_string(),
        };

        let mut report = Vec::new();

        for span in [
            Span::KeystrokeToPaint,
            Span::ScrollTickToPaint,
            Span::UiThreadTask,
            Span::HighlightUpdate,
        ] {
            let metric = span.metric();
            if let Some(value) = self.reduce(span, metric.statistic()) {
                report.push(measurement(metric, value.as_secs_f64() * 1000.0));
            }
        }

        if let Some(first_frame) = *self.first_frame.lock().expect("instrument lock") {
            report.push(measurement(Metric::ColdStart, first_frame.as_secs_f64() * 1000.0));
        }

        let peak_mb = self.peak_memory_kb.load(Ordering::Relaxed) as f64 / 1024.0;
        if peak_mb > 0.0 {
            report.push(measurement(Metric::PeakResidentMemory, peak_mb));
            report.push(measurement(Metric::TypicalResidentMemory, peak_mb));
            report.push(measurement(Metric::IdleResidentMemory, peak_mb));
        }

        let window = self.idle_window_nanos.load(Ordering::Relaxed);
        if window > 0 {
            let busy = self.idle_busy_nanos.load(Ordering::Relaxed) as f64;
            report.push(measurement(Metric::IdleCpu, (busy / window as f64) * 100.0));
        }

        report
    }

    /// Serialised for the runner that spawned this process. The product measures
    /// itself; the runner only collects.
    pub fn to_json(&self, runner: Runner, core_topology: &str, round_trip_ms: u32) -> String {
        let rows: Vec<String> = self
            .report(runner, core_topology, round_trip_ms)
            .iter()
            .map(|m| {
                format!(
                    "    {{ \"metric\": \"{}\", \"measured\": {:.4}, \"unit\": \"{}\" }}",
                    m.metric.name(),
                    m.measured,
                    m.metric.unit()
                )
            })
            .collect();
        format!("{{\n  \"measurements\": [\n{}\n  ]\n}}", rows.join(",\n"))
    }
}

impl vulcan_app::ports::frame_recorder::FrameRecorderPort for Instrument {
    fn observe(&self, metric: Metric, elapsed: Duration) {
        let span = match metric {
            Metric::KeystrokeToPaint => Span::KeystrokeToPaint,
            Metric::ScrollTickToPaint => Span::ScrollTickToPaint,
            Metric::LongestUiThreadTask => Span::UiThreadTask,
            Metric::HighlightUpdate => Span::HighlightUpdate,
            // Every other budget is measured somewhere other than a frame.
            _ => return,
        };
        self.observe(span, elapsed);
    }

    fn mark_first_frame(&self) {
        Instrument::mark_first_frame(self);
    }
}

/// Resident set size, read from the kernel rather than estimated.
///
/// Two implementations, because the kernels expose it differently and the
/// Linux one returned nothing on macOS — which left the memory budgets
/// unmeasured on the runner that is supposed to be authoritative.
#[cfg(target_os = "linux")]
fn resident_kb() -> Option<u64> {
    let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
    Some(pages * (page_size() / 1024))
}

/// `ps` reports resident size in kilobytes directly. Shelling out is not
/// elegant, but the alternative is `task_info` through `mach`, and a dependency
/// on a C binding for one number that this reads once a second is a poor trade.
#[cfg(target_os = "macos")]
fn resident_kb() -> Option<u64> {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn resident_kb() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn page_size() -> u64 {
    // 4 KiB everywhere this product targets; read it rather than assume when the
    // platform disagrees.
    4096
}

/// Processor time this process has consumed, across all its threads.
///
/// The idle budget is about work done, and an earlier version inferred it from
/// how far a sleep overran its deadline. That measures the scheduler's timer
/// granularity, not the product: on macOS it reported ten percent of a core for
/// a window that was doing nothing at all.
fn cpu_time() -> Option<Duration> {
    let out = std::process::Command::new("ps")
        .args(["-o", "time=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    parse_cpu_time(String::from_utf8_lossy(&out.stdout).trim())
}

/// `ps` prints processor time as `MM:SS.ss`, or `HH:MM:SS` once it is large.
pub fn parse_cpu_time(text: &str) -> Option<Duration> {
    let mut seconds = 0.0;
    for part in text.split(':') {
        seconds = seconds * 60.0 + part.parse::<f64>().ok()?;
    }
    Some(Duration::from_secs_f64(seconds))
}

/// Parses a report a measured process wrote, for the runner that collects it.
pub fn parse_report(text: &str) -> Vec<(String, f64)> {
    text.split("\"metric\":")
        .skip(1)
        .filter_map(|chunk| {
            let name = chunk.split('"').nth(1)?.to_string();
            let measured = chunk
                .split("\"measured\":")
                .nth(1)?
                .trim_start()
                .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                .next()?
                .parse()
                .ok()?;
            Some((name, measured))
        })
        .collect()
}
