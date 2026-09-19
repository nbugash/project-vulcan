//! Injecting round-trip latency for the measurement profiles.
//!
//! `round_trip_ms` was carried through every measurement and applied to nothing,
//! so a completion-popup figure taken at "80 ms" was a local figure wearing a
//! label. This applies the delay with `tc netem` on the loopback device and
//! reads it back; if it cannot, the run refuses rather than reporting a number
//! that describes a different network than the one it claims.

use std::process::Command;

use vulcan_app::ports::constrained_runner::MeasurementError;

const DEVICE: &str = "lo";

/// A delay held for as long as this value lives, and removed when it drops.
///
/// Tying removal to the value means an interrupted run cannot leave the machine
/// with a latency rule on its loopback device, which would silently corrupt
/// every later measurement on it.
pub struct InjectedLatency {
    round_trip_ms: u32,
}

impl InjectedLatency {
    /// `netem delay` applies to egress, so half the round trip in each
    /// direction. Zero is not a no-op: it clears any rule left behind.
    pub fn apply(round_trip_ms: u32) -> Result<Self, MeasurementError> {
        clear();
        if round_trip_ms == 0 {
            return Ok(Self { round_trip_ms });
        }

        let one_way = round_trip_ms as f64 / 2.0;
        let applied = tc(&["qdisc", "add", "dev", DEVICE, "root", "netem", "delay", &format!("{one_way}ms")]);
        if !applied {
            return Err(MeasurementError::ProductFailedToStart(format!(
                "cannot apply {round_trip_ms}ms round trip: `tc qdisc add dev {DEVICE} root netem` \
                 failed, which needs NET_ADMIN. Refusing to report a latency profile that was \
                 never in force"
            )));
        }

        match observed() {
            Some(seen) if (seen - one_way).abs() < 0.5 => Ok(Self { round_trip_ms }),
            Some(seen) => {
                clear();
                Err(MeasurementError::ProductFailedToStart(format!(
                    "asked for {one_way}ms each way, the device reports {seen}ms"
                )))
            }
            None => {
                clear();
                Err(MeasurementError::ProductFailedToStart(
                    "the delay was applied but cannot be read back".into(),
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

fn clear() {
    tc(&["qdisc", "del", "dev", DEVICE, "root"]);
}

fn tc(args: &[&str]) -> bool {
    Command::new("tc").args(args).output().map(|out| out.status.success()).unwrap_or(false)
}

/// The one-way delay the device reports, in milliseconds.
pub fn observed() -> Option<f64> {
    let out = Command::new("tc").args(["qdisc", "show", "dev", DEVICE]).output().ok()?;
    parse_delay(&String::from_utf8_lossy(&out.stdout))
}

/// `qdisc netem 8001: root refcnt 2 limit 1000 delay 40ms` -> 40.0
pub fn parse_delay(text: &str) -> Option<f64> {
    let after = text.split("delay ").nth(1)?;
    let value = after.split_whitespace().next()?;
    value.strip_suffix("ms").and_then(|n| n.parse().ok()).or_else(|| {
        value.strip_suffix("us").and_then(|n| n.parse::<f64>().ok()).map(|us| us / 1000.0)
    })
}
