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

/// Resident set size, from the kernel, without spawning anything.
///
/// An earlier macOS implementation shelled out to `ps`. `sample_memory` runs on
/// every observation, so a three hundred sample run forked several hundred
/// times, and the cost of forking is charged to the process being measured. It
/// reported a sleeping thread using two thirds of a core. A measurement that
/// consumes what it measures is not a measurement.
#[cfg(target_os = "linux")]
fn resident_kb() -> Option<u64> {
    let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
    Some(pages * 4)
}

#[cfg(target_os = "macos")]
fn resident_kb() -> Option<u64> {
    // mach_task_basic_info carries the resident size in bytes.
    let mut info = libc::mach_task_basic_info {
        virtual_size: 0,
        resident_size: 0,
        resident_size_max: 0,
        user_time: libc::time_value_t { seconds: 0, microseconds: 0 },
        system_time: libc::time_value_t { seconds: 0, microseconds: 0 },
        policy: 0,
        suspend_count: 0,
    };
    let mut count = (std::mem::size_of::<libc::mach_task_basic_info>()
        / std::mem::size_of::<libc::natural_t>()) as libc::mach_msg_type_number_t;

    // Safety: the task is our own, and the buffer and its length are matched.
    let status = unsafe {
        libc::task_info(
            libc::mach_task_self_,
            libc::MACH_TASK_BASIC_INFO,
            &mut info as *mut _ as libc::task_info_t,
            &mut count,
        )
    };
    (status == libc::KERN_SUCCESS).then(|| info.resident_size / 1024)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn resident_kb() -> Option<u64> {
    None
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
