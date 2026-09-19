//! Loading the previously accepted measurements.
//!
//! A baseline captured on a different core topology is rejected rather than
//! compared, because the regression rule would otherwise report scheduling
//! differences as product regressions.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaselineError {
    Unreadable(String),
    /// Present but not comparable. Not an error the gate fails on: the run
    /// proceeds with no baseline, so nothing is compared rather than compared
    /// wrongly.
    TopologyMismatch { expected: String, found: String },
}

pub struct Baseline {
    pub values: HashMap<String, f64>,
}

impl Baseline {
    pub fn empty() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn load(path: &std::path::Path, expected_topology: &str) -> Result<Self, BaselineError> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| BaselineError::Unreadable(error.to_string()))?;

        let found = field(&text, "core_topology").unwrap_or_default();
        if found != expected_topology {
            return Err(BaselineError::TopologyMismatch {
                expected: expected_topology.to_string(),
                found,
            });
        }

        let mut values = HashMap::new();
        for chunk in text.split("\"metric\":").skip(1) {
            if let (Some(metric), Some(measured)) = (quoted(chunk), number_after(chunk, "\"value\":")) {
                values.insert(metric, measured);
            }
        }
        Ok(Self { values })
    }
}

fn field(text: &str, key: &str) -> Option<String> {
    text.split(&format!("\"{key}\":")).nth(1).and_then(quoted)
}

fn quoted(text: &str) -> Option<String> {
    let start = text.find('"')? + 1;
    let rest = &text[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn number_after(text: &str, key: &str) -> Option<f64> {
    let rest = text.split(key).nth(1)?;
    let trimmed = rest.trim_start();
    let end = trimmed.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')?;
    trimmed[..end].parse().ok()
}
