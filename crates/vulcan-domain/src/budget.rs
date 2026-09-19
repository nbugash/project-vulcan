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
    FuzzyFileOpen,
    ProjectTextSearch,
    CompletionPopup,
    DiagnosticsAfterPause,
    IdleResidentMemory,
    TypicalResidentMemory,
    PeakResidentMemory,
    IdleCpu,
}

impl Metric {
    /// The thirteen budgets Principle VI defines.
    pub const ALL: [Metric; 13] = [
        Metric::KeystrokeToPaint,
        Metric::ScrollTickToPaint,
        Metric::LongestUiThreadTask,
        Metric::HighlightUpdate,
        Metric::ColdStart,
        Metric::FuzzyFileOpen,
        Metric::ProjectTextSearch,
        Metric::CompletionPopup,
        Metric::DiagnosticsAfterPause,
        Metric::IdleResidentMemory,
        Metric::TypicalResidentMemory,
        Metric::PeakResidentMemory,
        Metric::IdleCpu,
    ];

    pub fn budget(self) -> f64 {
        match self {
            Metric::KeystrokeToPaint | Metric::ScrollTickToPaint | Metric::LongestUiThreadTask => 8.0,
            Metric::HighlightUpdate => 16.0,
            Metric::ColdStart => 300.0,
            Metric::FuzzyFileOpen => 50.0,
            Metric::ProjectTextSearch => 500.0,
            Metric::CompletionPopup => 250.0,
            Metric::DiagnosticsAfterPause => 2000.0,
            Metric::IdleResidentMemory => 400.0,
            Metric::TypicalResidentMemory => 1500.0,
            Metric::PeakResidentMemory => 2500.0,
            Metric::IdleCpu => 1.0,
        }
    }

    pub fn unit(self) -> &'static str {
        match self {
            Metric::IdleResidentMemory | Metric::TypicalResidentMemory | Metric::PeakResidentMemory => "MB",
            Metric::IdleCpu => "%",
            _ => "ms",
        }
    }

    /// True when the budget is stated against a reference round trip rather than
    /// in absolute terms.
    pub fn round_trip_sensitive(self) -> bool {
        matches!(self, Metric::CompletionPopup | Metric::DiagnosticsAfterPause)
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
            Metric::FuzzyFileOpen => "FuzzyFileOpen",
            Metric::ProjectTextSearch => "ProjectTextSearch",
            Metric::CompletionPopup => "CompletionPopup",
            Metric::DiagnosticsAfterPause => "DiagnosticsAfterPause",
            Metric::IdleResidentMemory => "IdleResidentMemory",
            Metric::TypicalResidentMemory => "TypicalResidentMemory",
            Metric::PeakResidentMemory => "PeakResidentMemory",
            Metric::IdleCpu => "IdleCpu",
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
