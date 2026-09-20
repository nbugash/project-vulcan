//! Shared report emission for every gate.
//!
//! A report carries the inputs that make a verdict reproducible, so that a
//! reader can tell whether two runs are comparable at all.

use vulcan_domain::verdict::{GateError, GateVerdict};

pub struct Report<'a> {
    pub gate: &'a str,
    pub outcome: &'a Result<GateVerdict, GateError>,
    pub context: Vec<(String, String)>,
}

impl Report<'_> {
    pub fn to_json(&self) -> String {
        let (status, reason, findings) = match self.outcome {
            Ok(GateVerdict::Passed) => ("passed", String::new(), Vec::new()),
            Ok(GateVerdict::Regressed { findings }) => ("regressed", String::new(), findings.clone()),
            Ok(GateVerdict::Failed { findings }) => ("failed", String::new(), findings.clone()),
            Err(error) => ("could_not_judge", error.reason().to_string(), Vec::new()),
        };
        let mut json = String::from("{\n");
        json.push_str(&format!("  \"gate\": {},\n", quote(self.gate)));
        json.push_str(&format!("  \"status\": {},\n", quote(status)));
        if !reason.is_empty() {
            json.push_str(&format!("  \"reason\": {},\n", quote(&reason)));
        }
        for (key, value) in &self.context {
            json.push_str(&format!("  {}: {},\n", quote(key), quote(value)));
        }
        json.push_str("  \"findings\": [");
        json.push_str(
            &findings
                .iter()
                .map(|finding| format!("\n    {}", quote(finding)))
                .collect::<Vec<_>>()
                .join(","),
        );
        if findings.is_empty() { json.push_str("]\n}") } else { json.push_str("\n  ]\n}") }
        json
    }

    pub fn to_text(&self) -> String {
        match self.outcome {
            Ok(GateVerdict::Passed) => format!("PASS: {}", self.gate),
            Ok(GateVerdict::Regressed { findings }) => {
                format!("PASS (regressed): {}\n  {}", self.gate, findings.join("\n  "))
            }
            Ok(GateVerdict::Failed { findings }) => {
                format!("FAIL: {}\n  {}", self.gate, findings.join("\n  "))
            }
            Err(error) => format!("COULD NOT JUDGE: {} — {}", self.gate, error.reason()),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self.outcome {
            Ok(verdict) => verdict.exit_code(),
            Err(error) => error.exit_code(),
        }
    }
}

fn quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    format!("\"{escaped}\"")
}
pub mod discrepancies;
