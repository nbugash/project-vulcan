//! T040: in-process instrumentation reports exact figures, not samples.

use std::time::Duration;

use vulcan_adapters::measurement::instrument::{parse_report, Instrument, Span};
use vulcan_domain::budget::{Metric, Runner};

fn measurement(instrument: &Instrument, metric: Metric) -> Option<f64> {
    report(instrument)
        .into_iter()
        .find(|(candidate, _)| *candidate == metric)
        .map(|(_, measured)| measured)
}

fn report(instrument: &Instrument) -> Vec<(Metric, f64)> {
    instrument
        .report(Runner::LinuxCgroup, "6P+0E", 0)
        .into_iter()
        .map(|m| (m.metric, m.measured))
        .collect()
}

#[test]
fn a_recorded_span_reports_its_own_duration() {
    let instrument = Instrument::new();
    instrument.observe(Span::KeystrokeToPaint, Duration::from_micros(6200));
    let measured = report(&instrument)
        .into_iter()
        .find(|(metric, _)| *metric == Metric::KeystrokeToPaint)
        .expect("keystroke measured")
        .1;
    assert!((measured - 6.2).abs() < 0.001, "expected 6.2ms exactly, got {measured}");
}

#[test]
fn the_worst_frame_is_reported_not_the_average() {
    // Averaging would report 5ms here and hide the frame a user actually saw.
    let instrument = Instrument::new();
    for millis in [2, 3, 20, 2] {
        instrument.observe(Span::KeystrokeToPaint, Duration::from_millis(millis));
    }
    let measured = report(&instrument)
        .into_iter()
        .find(|(metric, _)| *metric == Metric::KeystrokeToPaint)
        .unwrap()
        .1;
    assert!((measured - 20.0).abs() < 0.5, "expected the worst frame, got {measured}");
}

#[test]
fn a_span_that_never_ran_is_absent_rather_than_zero() {
    // Reporting zero for something unmeasured would pass a budget it never met.
    let instrument = Instrument::new();
    instrument.observe(Span::KeystrokeToPaint, Duration::from_millis(1));
    assert!(!report(&instrument)
        .iter()
        .any(|(metric, _)| *metric == Metric::ScrollTickToPaint));
}

#[test]
fn cold_start_is_measured_to_the_first_frame() {
    let instrument = Instrument::new();
    std::thread::sleep(Duration::from_millis(12));
    instrument.mark_first_frame();
    let measured = report(&instrument)
        .into_iter()
        .find(|(metric, _)| *metric == Metric::ColdStart)
        .expect("cold start measured")
        .1;
    assert!(measured >= 12.0, "cold start must include the time before the frame: {measured}");
}

#[test]
fn a_later_frame_does_not_overwrite_cold_start() {
    let instrument = Instrument::new();
    instrument.mark_first_frame();
    std::thread::sleep(Duration::from_millis(15));
    instrument.mark_first_frame();
    let measured = report(&instrument)
        .into_iter()
        .find(|(metric, _)| *metric == Metric::ColdStart)
        .unwrap()
        .1;
    assert!(measured < 15.0, "cold start is the first frame, not the latest: {measured}");
}

#[test]
fn memory_is_read_from_the_kernel_and_is_plausible() {
    let instrument = Instrument::new();
    instrument.sample_memory();
    let measured = report(&instrument)
        .into_iter()
        .find(|(metric, _)| *metric == Metric::PeakResidentMemory)
        .expect("memory measured")
        .1;
    assert!(measured > 0.1 && measured < 8192.0, "implausible resident memory: {measured} MB");
}

#[test]
fn the_record_helper_times_the_work_and_returns_its_value() {
    let instrument = Instrument::new();
    let value = instrument.record(Span::UiThreadTask, || {
        std::thread::sleep(Duration::from_millis(5));
        41 + 1
    });
    assert_eq!(value, 42, "instrumenting a call site must not change its shape");
    let measured = report(&instrument)
        .into_iter()
        .find(|(metric, _)| *metric == Metric::LongestUiThreadTask)
        .unwrap()
        .1;
    assert!(measured >= 5.0, "the span must cover the work: {measured}");
}

#[test]
fn a_report_round_trips_through_json() {
    let instrument = Instrument::new();
    instrument.observe(Span::KeystrokeToPaint, Duration::from_micros(7100));
    instrument.mark_first_frame();
    let json = instrument.to_json(Runner::AppleSilicon, "2P+4E", 80);

    let parsed = parse_report(&json);
    let keystroke = parsed
        .iter()
        .find(|(name, _)| name == "KeystrokeToPaint")
        .expect("present after round trip");
    assert!((keystroke.1 - 7.1).abs() < 0.01, "value survived: {}", keystroke.1);
}





/// A p99 budget is not a maximum, and conflating them makes one hiccup from
/// another process decide the verdict.
#[test]
fn a_percentile_budget_ignores_a_single_outlier() {
    let instrument = Instrument::new();
    for _ in 0..199 {
        instrument.observe(Span::KeystrokeToPaint, Duration::from_millis(5));
    }
    // One frame lost to something else on the machine.
    instrument.observe(Span::KeystrokeToPaint, Duration::from_millis(90));

    let measured = measurement(&instrument, Metric::KeystrokeToPaint).expect("measured");
    assert!(measured < 10.0, "p99 of 200 samples reported {measured}ms; that is the outlier");
}

#[test]
fn the_longest_ui_thread_task_is_the_longest_one() {
    // This budget is a maximum by its own wording, so the outlier is the point.
    let instrument = Instrument::new();
    for _ in 0..199 {
        instrument.observe(Span::UiThreadTask, Duration::from_millis(2));
    }
    instrument.observe(Span::UiThreadTask, Duration::from_millis(40));

    let measured = measurement(&instrument, Metric::LongestUiThreadTask).expect("measured");
    assert!((measured - 40.0).abs() < 0.5, "expected the longest task, got {measured}ms");
}

/// Sampling must not cost anything the sampler would then report.
///
/// The macOS implementation used to shell out to `ps` on every observation. A
/// three hundred sample run forked several hundred times, fork is charged to
/// the parent, and a sleeping thread was measured using two thirds of a core.
#[test]
fn sampling_does_not_consume_what_it_measures() {
    let instrument = Instrument::new();

    let before = std::time::Instant::now();
    for _ in 0..2_000 {
        instrument.sample_memory();
    }
    let elapsed = before.elapsed();

    assert!(
        elapsed < Duration::from_millis(200),
        "2000 memory samples took {elapsed:?}; sampling is spawning something"
    );
}
