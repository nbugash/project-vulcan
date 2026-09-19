// T018: shared port contract suites.
//
// One suite per port, run against every implementation including the in-memory
// fake. Without this the fakes drift from the adapters and use-case tests pass
// against a fake no real adapter matches (Principle V).

use std::path::Path;
use vulcan_app::ports::workspace_graph::{GraphError, WorkspaceGraphPort};

/// Every `WorkspaceGraphPort` must satisfy these, fake and real alike.
pub fn workspace_graph_contract<P: WorkspaceGraphPort>(port: &P, valid_workspace: &Path) {
    let edges = port
        .read_edges(valid_workspace)
        .expect("a valid workspace yields edges");

    for edge in &edges {
        assert!(!edge.from.is_empty(), "every edge names its source crate");
        assert!(!edge.to.is_empty(), "every edge names its target crate");
        assert!(
            edge.declared_at.contains(':'),
            "every edge carries a manifest location, got {:?}",
            edge.declared_at
        );
    }

    // Reading twice yields the same graph: the port is a read, not a mutation.
    let again = port.read_edges(valid_workspace).expect("second read succeeds");
    assert_eq!(edges.len(), again.len(), "reads are stable");
}

/// An unreadable path is could-not-judge, never an empty graph, because an empty
/// graph would read as a passing gate.
pub fn workspace_graph_refuses_unreadable<P: WorkspaceGraphPort>(port: &P) {
    match port.read_edges(Path::new("/nonexistent/vulcan/workspace")) {
        Err(GraphError::Unparseable { .. }) | Err(GraphError::UndeclaredLayer { .. }) => {}
        Ok(edges) => panic!("expected an error, got {} edges", edges.len()),
    }
}

use vulcan_app::ports::constrained_runner::{
    ConstrainedRunnerPort, ConstraintError, MeasurementError,
};

/// Every `ConstrainedRunnerPort` must satisfy these, whether it can enforce its
/// constraints here or not. The contract is about shape, not about succeeding:
/// a runner that cannot enforce limits on this machine is still correct, as long
/// as it says so instead of measuring.
pub fn constrained_runner_contract<P: ConstrainedRunnerPort>(port: &P) {
    match port.assert_constraints() {
        Ok(topology) => {
            assert!(
                topology.performance > 0,
                "a runner that reports enforcement must report at least one performance core"
            );
            let rendered = topology.to_string();
            assert!(
                rendered.contains('P') && rendered.contains('E'),
                "topology must render as NP+ME so a report names the machine it measured: {rendered}"
            );
        }
        Err(ConstraintError::NotEnforceable(reason)) => {
            assert!(
                !reason.trim().is_empty(),
                "a refusal must say which constraint could not be applied, or nobody can fix it"
            );
        }
    }
}

/// A runner that could not enforce its constraints must not then hand back
/// measurements. Reporting numbers from an unconstrained machine is the failure
/// FR-005 exists to prevent, and it must be impossible per adapter, not only
/// prevented by the use case above it.
pub fn constrained_runner_refuses_to_measure_unconstrained<P: ConstrainedRunnerPort>(port: &P) {
    if port.assert_constraints().is_ok() {
        return; // Enforcement available here; this case does not apply.
    }
    match port.run_instrumented(0) {
        Err(MeasurementError::ProductFailedToStart(_)) | Err(MeasurementError::MetricUnavailable(_)) => {}
        Ok(measurements) => panic!(
            "constraints were not enforced, yet the runner returned {} measurements",
            measurements.len()
        ),
    }
}
