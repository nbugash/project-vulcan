//! Recording how long the interface took to do something.
//!
//! The shell is where frame and input timings actually happen, and the
//! instrument that stores them is an adapter. The shell may not depend on an
//! adapter — that is the edge gate 1 rejects — so it depends on this instead,
//! and the composition root decides whether anything is listening.

use std::time::Duration;

use vulcan_domain::budget::Metric;

pub trait FrameRecorderPort: Send + Sync {
    /// One observation of one metric. Called from the path being measured, so
    /// it must be cheap: a recorder that costs a millisecond has invalidated
    /// every figure it reports.
    fn observe(&self, metric: Metric, elapsed: Duration);

    /// The first frame the user could see, which ends cold start.
    fn mark_first_frame(&self);
}

/// What the shell uses when nobody is measuring, which is every ordinary run.
///
/// Recording is not conditional at the call site: the shell always reports, and
/// this throws the reports away. A measured path that only exists while
/// measuring is not the path the user runs.
#[derive(Debug, Clone, Copy, Default)]
pub struct NotRecording;

impl FrameRecorderPort for NotRecording {
    fn observe(&self, _metric: Metric, _elapsed: Duration) {}
    fn mark_first_frame(&self) {}
}
