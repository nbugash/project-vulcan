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

use vulcan_domain::budget::{BudgetMeasurement, Metric, Runner};

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

    pub fn sample_memory(&self) {
        if let Some(kb) = resident_kb() {
            self.peak_memory_kb.fetch_max(kb, Ordering::Relaxed);
        }
    }

    /// The worst case, not the average. A budget describes the frame a user
    /// notices, and averaging hides exactly that frame.
    fn worst(&self, span: Span) -> Option<Duration> {
        self.spans
            .lock()
            .expect("instrument lock")
            .iter()
            .filter(|(kind, _)| *kind == span)
            .map(|(_, elapsed)| *elapsed)
            .max()
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
            if let Some(worst) = self.worst(span) {
                report.push(measurement(span.metric(), worst.as_secs_f64() * 1000.0));
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
fn resident_kb() -> Option<u64> {
    let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
    Some(pages * (page_size() / 1024))
}

fn page_size() -> u64 {
    // 4 KiB everywhere this product targets; read it rather than assume when the
    // platform disagrees.
    4096
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
