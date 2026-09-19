//! Gate 1: judge the workspace dependency graph against declared layering.

use vulcan_domain::layer_edge::{Language, LayerEdge, LayerRule};
use vulcan_domain::verdict::{GateError, GateVerdict};

use crate::ports::workspace_graph::{GraphError, WorkspaceGraphPort};

pub struct CheckBoundaries<G: WorkspaceGraphPort> {
    graph: G,
}

impl<G: WorkspaceGraphPort> CheckBoundaries<G> {
    pub fn new(graph: G) -> Self {
        Self { graph }
    }

    pub fn execute(&self, manifest_path: &std::path::Path) -> Result<GateVerdict, GateError> {
        self.refuse_ungoverned_languages(manifest_path)?;

        let edges = self
            .graph
            .read_edges(manifest_path)
            .map_err(|error| GateError::CouldNotJudge(describe(&error)))?;

        let mut findings: Vec<String> = edges
            .iter()
            .filter(|edge| !LayerRule::is_permitted(edge))
            .map(LayerRule::describe_violation)
            .collect();

        findings.extend(cycles(&edges));

        if findings.is_empty() {
            Ok(GateVerdict::Passed)
        } else {
            Ok(GateVerdict::Failed { findings })
        }
    }

    /// FR-003. Refusing is exit 1, distinct from a failure: nothing was judged
    /// wrong, the gate simply cannot speak for this source.
    fn refuse_ungoverned_languages(&self, manifest_path: &std::path::Path) -> Result<(), GateError> {
        let extensions = self
            .graph
            .source_extensions(manifest_path)
            .map_err(|error| GateError::CouldNotJudge(describe(&error)))?;

        let mut ungoverned: Vec<String> = extensions
            .into_iter()
            .filter(|extension| {
                Language::of_extension(extension).is_none()
                    && !Language::ignored().contains(&extension.as_str())
            })
            .collect();
        ungoverned.sort();
        ungoverned.dedup();

        if ungoverned.is_empty() {
            return Ok(());
        }
        Err(GateError::CouldNotJudge(format!(
            "no boundary rule is defined for source of type: {}; \
             the gate governs {:?} and would otherwise report a pass it did not check",
            ungoverned.join(", "),
            Language::extensions()
        )))
    }
}

/// Reported separately from a reversed edge: a cycle is a different defect and
/// naming it as one saves the reader working it out from a list of edges.
fn cycles(edges: &[LayerEdge]) -> Vec<String> {
    let mut findings = Vec::new();
    for edge in edges {
        if edges.iter().any(|other| other.from == edge.to && other.to == edge.from) && edge.from < edge.to {
            findings.push(format!(
                "dependency cycle between {} and {} [{}]",
                edge.from, edge.to, edge.declared_at
            ));
        }
    }
    findings
}

fn describe(error: &GraphError) -> String {
    match error {
        GraphError::Unparseable { path, detail } => format!("cannot parse {path}: {detail}"),
        GraphError::UndeclaredLayer { crate_name, path } => format!(
            "{crate_name} declares no layer in {path}; add [package.metadata.vulcan] layer"
        ),
    }
}
