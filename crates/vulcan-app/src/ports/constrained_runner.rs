use vulcan_domain::budget::BudgetMeasurement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreTopology {
    pub performance: u8,
    pub efficiency: u8,
}

impl std::fmt::Display for CoreTopology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}P+{}E", self.performance, self.efficiency)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintError {
    /// Limits were requested but not enforced. Measuring anyway would report a
    /// machine the product does not target (FR-005).
    NotEnforceable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeasurementError {
    ProductFailedToStart(String),
    /// A metric that cannot be measured fails the run rather than being omitted,
    /// so a report is never quietly partial.
    MetricUnavailable(String),
}

pub trait ConstrainedRunnerPort {
    fn assert_constraints(&self) -> Result<CoreTopology, ConstraintError>;
    fn run_instrumented(&self, round_trip_ms: u32) -> Result<Vec<BudgetMeasurement>, MeasurementError>;
}
