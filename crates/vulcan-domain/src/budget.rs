//! Budget metrics and the verdict rule applied to a measurement.
//!
//! Budgets come from the constitution and are compiled in, so a failing build
//! cannot be fixed by editing a threshold.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    KeystrokeToPaint,
    ScrollTickToPaint,
    LongestUiThreadTask,
    HighlightUpdate,
    ColdStart,
    WarmStart,
    FuzzyFileOpen,
    ProjectTextSearch,
    CompletionPopup,
    DiagnosticsAfterPause,
    IdleResidentMemory,
    TypicalResidentMemory,
    PeakResidentMemory,
}

/// How a run of samples is reduced to the one figure a budget is judged against.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Statistic {
    Max,
    Percentile(f64),
}

impl Statistic {
    pub fn describe(self) -> String {
        match self {
            Statistic::Max => "max".to_string(),
            Statistic::Percentile(p) => format!("p{}", p as u32),
        }
    }
}

impl Metric {
    /// The thirteen budgets Principle VI defines.
    pub const ALL: [Metric; 13] = [
        Metric::KeystrokeToPaint,
        Metric::ScrollTickToPaint,
        Metric::LongestUiThreadTask,
        Metric::HighlightUpdate,
        Metric::ColdStart,
        Metric::WarmStart,
        Metric::FuzzyFileOpen,
        Metric::ProjectTextSearch,
        Metric::CompletionPopup,
        Metric::DiagnosticsAfterPause,
        Metric::IdleResidentMemory,
        Metric::TypicalResidentMemory,
        Metric::PeakResidentMemory,
    ];

    pub fn budget(self) -> f64 {
        match self {
            Metric::KeystrokeToPaint | Metric::ScrollTickToPaint | Metric::LongestUiThreadTask => 8.0,
            Metric::HighlightUpdate => 16.0,
            Metric::ColdStart => 300.0,
            // A launch with the binary and its libraries already in the page
            // cache. Provisional: set from measurement rather than intent, and
            // narrower than cold start because the work it excludes is exactly
            // the work a second launch does not repeat.
            Metric::WarmStart => 150.0,
            Metric::FuzzyFileOpen => 50.0,
            Metric::ProjectTextSearch => 500.0,
            Metric::CompletionPopup => 250.0,
            Metric::DiagnosticsAfterPause => 2000.0,
            Metric::IdleResidentMemory => 400.0,
            Metric::TypicalResidentMemory => 1500.0,
            Metric::PeakResidentMemory => 2500.0,
        }
    }

    pub fn unit(self) -> &'static str {
        match self {
            Metric::IdleResidentMemory | Metric::TypicalResidentMemory | Metric::PeakResidentMemory => "MB",
            _ => "ms",
        }
    }

    /// True when the budget is stated against a reference round trip rather than
    /// in absolute terms.
    pub fn round_trip_sensitive(self) -> bool {
        matches!(self, Metric::CompletionPopup | Metric::DiagnosticsAfterPause)
    }

    /// Which statistic over a run of samples this budget is stated against.
    ///
    /// The constitution is specific and the distinction matters: "8 ms p99" is
    /// not "no sample above 8 ms". Reporting the maximum for a p99 budget makes
    /// the verdict hostage to one hiccup from another process, which is what
    /// two runs on the same Mac minutes apart demonstrated — 3.97 ms and then
    /// 16.88 ms for the same work.
    pub fn statistic(self) -> Statistic {
        match self {
            // "Longest task on the UI thread" is a maximum by its own wording.
            Metric::LongestUiThreadTask => Statistic::Max,
            Metric::CompletionPopup => Statistic::Percentile(95.0),
            Metric::KeystrokeToPaint | Metric::ScrollTickToPaint => Statistic::Percentile(99.0),
            _ => Statistic::Max,
        }
    }

    /// Whether anything in the product can produce this metric yet.
    ///
    /// A budget for work that does not exist cannot be measured, and demanding
    /// it means the gate refuses forever: the shell has no scrolling, no
    /// completion engine, no index and no search, so five of these describe
    /// features later features will build. They are still budgets, and they
    /// still bind once their feature exists; they are simply not evidence
    /// against a shell that cannot exercise them.
    pub fn measurable_by_the_shell(self) -> bool {
        !matches!(
            self,
            Metric::ScrollTickToPaint
                | Metric::HighlightUpdate
                | Metric::CompletionPopup
                | Metric::FuzzyFileOpen
                | Metric::ProjectTextSearch
                | Metric::DiagnosticsAfterPause
        )
    }

    pub fn name(self) -> &'static str {
        match self {
            Metric::KeystrokeToPaint => "KeystrokeToPaint",
            Metric::ScrollTickToPaint => "ScrollTickToPaint",
            Metric::LongestUiThreadTask => "LongestUiThreadTask",
            Metric::HighlightUpdate => "HighlightUpdate",
            Metric::ColdStart => "ColdStart",
            Metric::WarmStart => "WarmStart",
            Metric::FuzzyFileOpen => "FuzzyFileOpen",
            Metric::ProjectTextSearch => "ProjectTextSearch",
            Metric::CompletionPopup => "CompletionPopup",
            Metric::DiagnosticsAfterPause => "DiagnosticsAfterPause",
            Metric::IdleResidentMemory => "IdleResidentMemory",
            Metric::TypicalResidentMemory => "TypicalResidentMemory",
            Metric::PeakResidentMemory => "PeakResidentMemory",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Regressed,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runner {
    AppleSilicon,
    LinuxCgroup,
}

impl Runner {
    /// Apple Silicon reproduces the baseline's performance and efficiency core
    /// split; the Linux runner is a second signal on the platform we also ship.
    pub fn is_authoritative(self) -> bool {
        matches!(self, Runner::AppleSilicon)
    }
}

#[derive(Debug, Clone)]
pub struct BudgetMeasurement {
    pub metric: Metric,
    pub measured: f64,
    pub round_trip_ms: u32,
    pub runner: Runner,
    pub core_topology: String,
}

impl BudgetMeasurement {
    /// Ten percent worse than an accepted baseline is reported for explanation
    /// even when it remains within budget.
    pub fn judge(&self, baseline: Option<f64>) -> Verdict {
        if self.measured > self.metric.budget() {
            return Verdict::Fail;
        }
        match baseline {
            Some(previous) if previous > 0.0 && self.measured > previous * 1.10 => Verdict::Regressed,
            _ => Verdict::Pass,
        }
    }
}
