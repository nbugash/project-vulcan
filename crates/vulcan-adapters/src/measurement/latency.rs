//! Injecting round-trip latency for the measurement profiles.
//!
//! Budgets involving a remote host are stated relative to round trip, so the
//! measurement produces one rather than depending on a network. `round_trip_ms`
//! used to be threaded through every measurement and applied to nothing, which
//! made an 80 ms figure a local figure wearing a label.
//!
//! Two backends, because the tools differ: `tc netem` on Linux, `dnctl` and
//! `pfctl` on macOS. Where neither is available the run refuses; a profile that
//! was never in force must not produce a number.

use std::process::Command;

use vulcan_app::ports::constrained_runner::MeasurementError;

/// The round trips the budgets are stated against.
pub const PROFILES: [u32; 4] = [0, 10, 30, 80];

/// Loopback, so a run is deterministic and the internet is not a variable.
#[cfg(target_os = "linux")]
const DEVICE: &str = "lo";

/// Our own pf anchor, so the machine's existing firewall rules are left alone.
/// Replacing the whole ruleset to measure latency would be a rude thing for a
/// build to do to a developer's machine.
#[cfg(target_os = "macos")]
const ANCHOR: &str = "vulcan.budget";
#[cfg(target_os = "macos")]
const PIPE: &str = "1";

/// A delay held for as long as this value lives, and removed when it drops.
///
/// Tying removal to the value means an interrupted run cannot leave a rule
/// behind to corrupt every later measurement on the machine.
pub struct InjectedLatency {
    round_trip_ms: u32,
}

impl InjectedLatency {
    /// The delay applies in each direction, so each way carries half the round
    /// trip. Zero is not a no-op: it clears anything left behind.
    pub fn apply(round_trip_ms: u32) -> Result<Self, MeasurementError> {
        clear();
        if round_trip_ms == 0 {
            return Ok(Self { round_trip_ms });
        }

        let one_way = round_trip_ms as f64 / 2.0;
        if !apply_one_way(one_way) {
            return Err(MeasurementError::ProductFailedToStart(unavailable(round_trip_ms)));
        }

        match observed() {
            Some(seen) if (seen - one_way).abs() < 0.5 => Ok(Self { round_trip_ms }),
            Some(seen) => {
                clear();
                Err(MeasurementError::ProductFailedToStart(format!(
                    "asked for {one_way}ms each way, the system reports {seen}ms"
                )))
            }
            None => {
                clear();
                Err(MeasurementError::ProductFailedToStart(
                    "the delay was applied but cannot be read back, so it cannot be trusted".into(),
                ))
            }
        }
    }

    pub fn round_trip_ms(&self) -> u32 {
        self.round_trip_ms
    }
}

impl Drop for InjectedLatency {
    fn drop(&mut self) {
        if self.round_trip_ms > 0 {
            clear();
        }
    }
}

fn unavailable(round_trip_ms: u32) -> String {
    let tool = if cfg!(target_os = "macos") { "dnctl and pfctl, which need sudo" } else { "tc netem, which needs NET_ADMIN" };
    format!(
        "cannot apply a {round_trip_ms}ms round trip: {tool} is unavailable here. \
         Refusing to report a latency profile that was never in force"
    )
}

// ---- Linux -------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn apply_one_way(one_way: f64) -> bool {
    run("tc", &["qdisc", "add", "dev", DEVICE, "root", "netem", "delay", &format!("{one_way}ms")])
}

#[cfg(target_os = "linux")]
fn clear() {
    run("tc", &["qdisc", "del", "dev", DEVICE, "root"]);
}

#[cfg(target_os = "linux")]
pub fn observed() -> Option<f64> {
    let out = Command::new("tc").args(["qdisc", "show", "dev", DEVICE]).output().ok()?;
    parse_delay(&String::from_utf8_lossy(&out.stdout))
}

// ---- macOS -------------------------------------------------------------------
//
// dummynet does the delaying and pf decides what enters the pipe. The rule goes
// in an anchor of our own so the existing ruleset is untouched, and pf is only
// enabled, never reconfigured wholesale.

#[cfg(target_os = "macos")]
fn apply_one_way(one_way: f64) -> bool {
    if !run("dnctl", &["pipe", PIPE, "config", "delay", &format!("{one_way}ms")]) {
        return false;
    }
    let rule = format!("dummynet in  quick on lo0 all pipe {PIPE}\ndummynet out quick on lo0 all pipe {PIPE}\n");
    if !pipe_into("pfctl", &["-a", ANCHOR, "-f", "-"], &rule) {
        return false;
    }
    // Already-enabled is not a failure; pf reports it on stderr and exits non-zero.
    run("pfctl", &["-E"]);
    true
}

#[cfg(target_os = "macos")]
fn clear() {
    run("pfctl", &["-a", ANCHOR, "-F", "all"]);
    run("dnctl", &["pipe", PIPE, "delete"]);
}

#[cfg(target_os = "macos")]
pub fn observed() -> Option<f64> {
    let out = Command::new("dnctl").args(["pipe", "show"]).output().ok()?;
    parse_delay(&String::from_utf8_lossy(&out.stdout))
}

// ---- Neither -----------------------------------------------------------------

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn apply_one_way(_one_way: f64) -> bool {
    false
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn clear() {}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn observed() -> Option<f64> {
    None
}

// ---- Shared ------------------------------------------------------------------

#[allow(dead_code)]
fn run(program: &str, args: &[&str]) -> bool {
    Command::new(program).args(args).output().map(|out| out.status.success()).unwrap_or(false)
}

#[allow(dead_code)]
fn pipe_into(program: &str, args: &[&str], stdin: &str) -> bool {
    use std::io::Write;
    use std::process::Stdio;

    let Ok(mut child) = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    if let Some(mut input) = child.stdin.take() {
        let _ = input.write_all(stdin.as_bytes());
    }
    child.wait().map(|status| status.success()).unwrap_or(false)
}

/// Reads a delay out of what the tool reports.
///
/// `tc`:    `qdisc netem 8001: root refcnt 2 limit 1000 delay 40ms`
/// `dnctl`: `00001:  10.000ms    0 ms burst 0` — the delay carries the unit.
pub fn parse_delay(text: &str) -> Option<f64> {
    let after = text.split("delay ").nth(1).or_else(|| {
        // dnctl prints the delay before the word, as `40.000ms`.
        text.split_whitespace().find(|word| word.ends_with("ms") && word.len() > 2)
    })?;
    let value = after.split_whitespace().next()?;
    value
        .strip_suffix("ms")
        .and_then(|n| n.parse().ok())
        .or_else(|| value.strip_suffix("us").and_then(|n| n.parse::<f64>().ok()).map(|us| us / 1000.0))
}
