//! T115: the latency profile is applied, not just recorded.

use vulcan_adapters::measurement::latency::{parse_delay, InjectedLatency};

#[test]
fn a_delay_is_read_back_from_what_the_device_reports() {
    assert_eq!(
        parse_delay("qdisc netem 8001: root refcnt 2 limit 1000 delay 40ms"),
        Some(40.0)
    );
    assert_eq!(parse_delay("qdisc netem 8001: root delay 5ms 1ms"), Some(5.0));
}

#[test]
fn microseconds_are_understood() {
    // `tc` reports sub-millisecond delays in microseconds.
    assert_eq!(parse_delay("qdisc netem 8001: root delay 500us"), Some(0.5));
}

#[test]
fn a_device_with_no_delay_reports_none() {
    assert_eq!(parse_delay("qdisc noqueue 0: root refcnt 2"), None);
}

#[test]
fn a_profile_that_cannot_be_applied_refuses_rather_than_reporting() {
    // Without NET_ADMIN this must fail. Reporting an 80ms figure from a machine
    // with no delay on it is the failure this test exists to prevent; the only
    // acceptable outcomes are a real delay or a refusal.
    match InjectedLatency::apply(80) {
        Ok(applied) => {
            // Privileged environment: the delay must really be in force.
            let seen = vulcan_adapters::measurement::latency::observed();
            assert_eq!(applied.round_trip_ms(), 80);
            assert!(seen.is_some(), "applied, but the device reports no delay");
        }
        Err(reason) => {
            let text = format!("{reason:?}");
            assert!(text.contains("80ms") || text.contains("NET_ADMIN"), "{text}");
        }
    }
}

#[test]
fn the_zero_profile_needs_no_privilege() {
    assert!(InjectedLatency::apply(0).is_ok(), "0ms is the absence of a delay");
}

/// The macOS backend reports delays in dnctl's format, which differs from tc's.
#[test]
fn a_dnctl_pipe_listing_is_understood() {
    assert_eq!(parse_delay("00001:  40.000ms    0 ms burst 0"), Some(40.0));
}

#[test]
fn the_profiles_are_the_ones_the_budgets_are_stated_against() {
    use vulcan_adapters::measurement::latency::PROFILES;
    assert_eq!(PROFILES, [0, 10, 30, 80]);
}

/// dnctl separates the number from its unit, which the first parser missed —
/// it applied a delay on a Mac and then refused because it could not read it
/// back.
#[test]
fn dnctl_prints_the_unit_as_a_separate_word() {
    assert_eq!(parse_delay("00001: unlimited    10 ms burst 0"), Some(10.0));
    assert_eq!(parse_delay("00001: unlimited     5 ms burst 0"), Some(5.0));
}

#[test]
fn a_pipe_with_no_delay_reports_none() {
    assert_eq!(parse_delay("00001: unlimited     0 ms burst 0"), Some(0.0));
    assert_eq!(parse_delay("00001: unlimited  burst 0"), None);
}
