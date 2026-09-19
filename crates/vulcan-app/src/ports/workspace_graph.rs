use vulcan_domain::layer_edge::LayerEdge;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    Unparseable { path: String, detail: String },
    /// A crate with no declared layer fails rather than defaulting, so adding a
    /// crate is a deliberate act.
    UndeclaredLayer { crate_name: String, path: String },
}

pub trait WorkspaceGraphPort {
    fn read_edges(&self, manifest_path: &std::path::Path) -> Result<Vec<LayerEdge>, GraphError>;

    /// File extensions found under the governed source roots, deduplicated.
    /// The use case decides which of them it has a rule for.
    fn source_extensions(&self, manifest_path: &std::path::Path) -> Result<Vec<String>, GraphError>;
}
