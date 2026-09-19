//! Inbound-adapter shell shared by every gate binary.
//!
//! Argument parsing and exit codes only; no gate logic. The three exit codes are
//! mapped here once, so no binary can accidentally report a gate that did not
//! run as one that passed.

use vulcan_adapters::reporting::Report;
use vulcan_domain::verdict::{GateError, GateVerdict};

pub struct Args {
    pub json: bool,
    pub report_path: Option<String>,
    pub positional: Vec<String>,
    pub flags: Vec<(String, String)>,
}

impl Args {
    pub fn parse<I: Iterator<Item = String>>(raw: I) -> Self {
        let mut json = false;
        let mut report_path = None;
        let mut positional = Vec::new();
        let mut flags = Vec::new();
        let mut items = raw.peekable();

        while let Some(item) = items.next() {
            match item.as_str() {
                "--json" => json = true,
                "--report" => report_path = items.next(),
                flag if flag.starts_with("--") => {
                    let value = items.next_if(|next| !next.starts_with("--")).unwrap_or_default();
                    flags.push((flag.trim_start_matches("--").to_string(), value));
                }
                other => positional.push(other.to_string()),
            }
        }
        Self { json, report_path, positional, flags }
    }

    pub fn flag(&self, name: &str) -> Option<&str> {
        self.flags.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
    }
}

/// Emits the report and returns the process exit code.
pub fn finish(gate: &str, args: &Args, outcome: Result<GateVerdict, GateError>, context: Vec<(String, String)>) -> i32 {
    let report = Report { gate, outcome: &outcome, context };
    let rendered = if args.json { report.to_json() } else { report.to_text() };
    println!("{rendered}");

    // A gate that could not judge writes no report file. The absence is the
    // signal: a file at the requested path always describes a run that happened,
    // so a later reader never mistakes a refusal for a result.
    if let (Some(path), Ok(_)) = (&args.report_path, &outcome) {
        // The caller asked for a report at a path; creating the directory it
        // names is part of honouring that, not something every caller should
        // have to arrange first.
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(error) = std::fs::create_dir_all(parent) {
                    eprintln!("warning: could not create {}: {error}", parent.display());
                }
            }
        }
        if let Err(error) = std::fs::write(path, report.to_json()) {
            eprintln!("warning: could not write report to {path}: {error}");
        }
    }
    report.exit_code()
}
