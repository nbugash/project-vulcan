//! Gate outcomes and their exit codes.
//!
//! A gate either judged and passed, judged and failed, or could not judge. The
//! third is a distinct outcome rather than a weaker second, because a check that
//! did not run has not passed.

/// The outcome of a gate that was able to judge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateVerdict {
    Passed,
    /// Under budget or within rule, but worse than the accepted baseline.
    /// Reported for explanation; does not block.
    Regressed { findings: Vec<String> },
    Failed { findings: Vec<String> },
}

impl GateVerdict {
    pub fn exit_code(&self) -> i32 {
        match self {
            GateVerdict::Passed | GateVerdict::Regressed { .. } => 0,
            GateVerdict::Failed { .. } => 2,
        }
    }

    pub fn findings(&self) -> &[String] {
        match self {
            GateVerdict::Passed => &[],
            GateVerdict::Regressed { findings } | GateVerdict::Failed { findings } => findings,
        }
    }

    pub fn is_blocking(&self) -> bool {
        matches!(self, GateVerdict::Failed { .. })
    }
}

/// A gate that could not reach a verdict. Never carries a passing outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    /// The gate could not run: a missing prerequisite, an unenforceable
    /// constraint, or an environment it refuses to judge from.
    CouldNotJudge(String),
}

impl GateError {
    pub fn exit_code(&self) -> i32 {
        1
    }

    pub fn reason(&self) -> &str {
        match self {
            GateError::CouldNotJudge(reason) => reason,
        }
    }
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not judge: {}", self.reason())
    }
}

impl std::error::Error for GateError {}
