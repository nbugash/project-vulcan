//! Round-trip injection on loopback.
//!
//! Budgets that involve a remote host are stated relative to round trip, so the
//! measurement produces one on demand. Injecting locally keeps runs deterministic
//! and removes the internet as a source of variance.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetemError {
    Unavailable(String),
    Failed(String),
}

pub const PROFILES: [u32; 4] = [0, 10, 30, 80];

pub struct Netem {
    interface: String,
}

impl Netem {
    pub fn new(interface: impl Into<String>) -> Self {
        Self { interface: interface.into() }
    }

    pub fn available() -> Result<(), NetemError> {
        which_tc().map(|_| ())
    }

    /// Applies a one-way delay of half the round trip, so a packet and its reply
    /// together cost the requested figure.
    pub fn apply(&self, round_trip_ms: u32) -> Result<(), NetemError> {
        let tc = which_tc()?;
        let _ = self.clear();
        if round_trip_ms == 0 {
            return Ok(());
        }
        let delay = format!("{}ms", round_trip_ms as f64 / 2.0);
        run(&tc, &["qdisc", "add", "dev", &self.interface, "root", "netem", "delay", &delay])
    }

    pub fn clear(&self) -> Result<(), NetemError> {
        let tc = which_tc()?;
        run(&tc, &["qdisc", "del", "dev", &self.interface, "root"])
    }
}

fn which_tc() -> Result<String, NetemError> {
    for candidate in ["/usr/sbin/tc", "/sbin/tc", "tc"] {
        if std::process::Command::new(candidate).arg("-help").output().is_ok() {
            return Ok(candidate.to_string());
        }
    }
    Err(NetemError::Unavailable("tc with netem is not available".into()))
}

fn run(tc: &str, args: &[&str]) -> Result<(), NetemError> {
    let output = std::process::Command::new(tc)
        .args(args)
        .output()
        .map_err(|error| NetemError::Failed(error.to_string()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(NetemError::Failed(String::from_utf8_lossy(&output.stderr).trim().to_string()))
    }
}
