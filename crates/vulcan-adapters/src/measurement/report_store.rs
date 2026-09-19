//! Measurement reports.
//!
//! A report records the inputs that make a verdict comparable: the runner, its
//! authority, and the observed core topology. A measurement is only comparable
//! to one taken on the same topology.

use vulcan_domain::budget::{BudgetMeasurement, Verdict};

pub struct MeasurementReport {
    pub runner: String,
    pub authoritative: bool,
    pub core_topology: String,
    pub measurements: Vec<(BudgetMeasurement, Verdict)>,
}

impl MeasurementReport {
    pub fn to_json(&self) -> String {
        let rows: Vec<String> = self
            .measurements
            .iter()
            .map(|(measurement, verdict)| {
                format!(
                    "    {{\n      \"metric\": \"{}\",\n      \"measured\": {{ \"value\": {:.3}, \"unit\": \"{}\" }},\n      \"budget\": {{ \"value\": {:.3}, \"unit\": \"{}\" }},\n      \"round_trip_ms\": {},\n      \"verdict\": \"{}\"\n    }}",
                    measurement.metric.name(),
                    measurement.measured,
                    measurement.metric.unit(),
                    measurement.metric.budget(),
                    measurement.metric.unit(),
                    measurement.round_trip_ms,
                    match verdict {
                        Verdict::Pass => "Pass",
                        Verdict::Regressed => "Regressed",
                        Verdict::Fail => "Fail",
                    }
                )
            })
            .collect();

        format!(
            "{{\n  \"runner\": \"{}\",\n  \"authoritative\": {},\n  \"core_topology\": \"{}\",\n  \"measurements\": [\n{}\n  ]\n}}",
            self.runner,
            self.authoritative,
            self.core_topology,
            rows.join(",\n")
        )
    }

    pub fn write(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_json())
    }
}
