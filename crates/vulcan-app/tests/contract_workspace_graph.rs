//! T024: the workspace graph contract, run against the fake and the real adapter.

mod contract {
    include!("contract/mod.rs");
}

use std::path::{Path, PathBuf};
use vulcan_app::ports::workspace_graph::{GraphError, WorkspaceGraphPort};
use vulcan_domain::layer_edge::{Layer, LayerEdge};

/// The in-memory fake used by use-case tests. It passes the same suite the real
/// adapter does, which is what makes those tests meaningful.
struct FakeGraph;

impl WorkspaceGraphPort for FakeGraph {
    fn source_extensions(&self, _: &std::path::Path) -> Result<Vec<String>, GraphError> {
        Ok(vec!["rs".to_string()])
    }

    fn read_edges(&self, path: &Path) -> Result<Vec<LayerEdge>, GraphError> {
        if !path.exists() {
            return Err(GraphError::Unparseable {
                path: path.display().to_string(),
                detail: "no such workspace".into(),
            });
        }
        Ok(vec![LayerEdge {
            from: "vulcan-app".into(),
            to: "vulcan-domain".into(),
            from_layer: Layer::Application,
            to_layer: Layer::Domain,
            declared_at: "crates/vulcan-app/Cargo.toml:9".into(),
        }])
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

#[test]
fn fake_satisfies_the_contract() {
    contract::workspace_graph_contract(&FakeGraph, &repository_root());
    contract::workspace_graph_refuses_unreadable(&FakeGraph);
}
