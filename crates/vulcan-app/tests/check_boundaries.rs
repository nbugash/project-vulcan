//! T023: the boundary use case, against a fake graph port.

use std::path::Path;
use vulcan_app::ports::workspace_graph::{GraphError, WorkspaceGraphPort};
use vulcan_app::use_cases::check_boundaries::CheckBoundaries;
use vulcan_domain::layer_edge::{Layer, LayerEdge};
use vulcan_domain::verdict::{GateError, GateVerdict};

struct FakeGraph(Result<Vec<LayerEdge>, GraphError>);

impl WorkspaceGraphPort for FakeGraph {
    fn source_extensions(&self, _: &std::path::Path) -> Result<Vec<String>, GraphError> {
        Ok(vec!["rs".to_string()])
    }

    fn read_edges(&self, _: &Path) -> Result<Vec<LayerEdge>, GraphError> {
        self.0.clone()
    }
}

fn edge(from: &str, to: &str, from_layer: Layer, to_layer: Layer) -> LayerEdge {
    LayerEdge {
        from: from.into(),
        to: to.into(),
        from_layer,
        to_layer,
        declared_at: format!("crates/{from}/Cargo.toml:9"),
    }
}

#[test]
fn a_clean_graph_passes() {
    let use_case = CheckBoundaries::new(FakeGraph(Ok(vec![
        edge("vulcan-app", "vulcan-domain", Layer::Application, Layer::Domain),
        edge("vulcan-adapters", "vulcan-app", Layer::Adapters, Layer::Application),
    ])));
    assert_eq!(use_case.execute(Path::new(".")).unwrap(), GateVerdict::Passed);
}

#[test]
fn every_offending_edge_is_named() {
    let use_case = CheckBoundaries::new(FakeGraph(Ok(vec![
        edge("vulcan-domain", "vulcan-adapters", Layer::Domain, Layer::Adapters),
        edge("vulcan-app", "vulcan-cli", Layer::Application, Layer::Composition),
        edge("vulcan-adapters", "vulcan-app", Layer::Adapters, Layer::Application),
    ])));
    let verdict = use_case.execute(Path::new(".")).unwrap();
    assert!(verdict.is_blocking());
    assert_eq!(verdict.findings().len(), 2);
    assert!(verdict.findings().iter().any(|f| f.contains("vulcan-domain")));
    assert!(verdict.findings().iter().any(|f| f.contains("vulcan-app")));
}

#[test]
fn a_violation_carries_its_manifest_location() {
    let use_case = CheckBoundaries::new(FakeGraph(Ok(vec![edge(
        "vulcan-domain",
        "vulcan-adapters",
        Layer::Domain,
        Layer::Adapters,
    )])));
    let verdict = use_case.execute(Path::new(".")).unwrap();
    assert!(verdict.findings()[0].contains("crates/vulcan-domain/Cargo.toml:9"));
}

#[test]
fn an_undeclared_layer_cannot_be_judged() {
    let use_case = CheckBoundaries::new(FakeGraph(Err(GraphError::UndeclaredLayer {
        crate_name: "vulcan-new".into(),
        path: "crates/vulcan-new/Cargo.toml".into(),
    })));
    // Could-not-judge, not failed: the difference is what stops a missing
    // declaration from reading as a passing gate.
    match use_case.execute(Path::new(".")) {
        Err(GateError::CouldNotJudge(reason)) => assert!(reason.contains("vulcan-new")),
        other => panic!("expected CouldNotJudge, got {other:?}"),
    }
}

#[test]
fn a_cycle_is_reported_as_a_cycle() {
    let use_case = CheckBoundaries::new(FakeGraph(Ok(vec![
        edge("a", "b", Layer::Adapters, Layer::Adapters),
        edge("b", "a", Layer::Adapters, Layer::Adapters),
    ])));
    let verdict = use_case.execute(Path::new(".")).unwrap();
    assert!(verdict.findings().iter().any(|f| f.contains("cycle")));
}

/// T104 / FR-003: a language with no rule is refused, not passed.
#[test]
fn source_in_an_ungoverned_language_is_refused_rather_than_passed() {
    struct PolyglotGraph;
    impl WorkspaceGraphPort for PolyglotGraph {
        fn source_extensions(&self, _: &std::path::Path) -> Result<Vec<String>, GraphError> {
            Ok(vec!["rs".into(), "go".into(), "md".into()])
        }
        fn read_edges(&self, _: &std::path::Path) -> Result<Vec<LayerEdge>, GraphError> {
            panic!("must refuse before reading any edge");
        }
    }

    match CheckBoundaries::new(PolyglotGraph).execute(std::path::Path::new(".")) {
        Err(GateError::CouldNotJudge(reason)) => {
            assert!(reason.contains("go"), "{reason}");
            assert!(!reason.contains("md"), "documentation carries no layering: {reason}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}
