//! Layering, expressed as the workspace dependency graph.
//!
//! Layers are crates, so most violations are already compile errors. These types
//! describe the edge the compiler accepts: a declared dependency pointing the
//! wrong way through the workspace.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    Domain,
    Application,
    Adapters,
    Composition,
    Tooling,
}

impl Layer {
    /// Parsed from `[package.metadata.vulcan] layer`. A crate with no declared
    /// layer is a failure rather than a default, so adding a crate is deliberate.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "Domain" => Some(Layer::Domain),
            "Application" => Some(Layer::Application),
            "Adapters" => Some(Layer::Adapters),
            "Composition" => Some(Layer::Composition),
            "Tooling" => Some(Layer::Tooling),
            _ => None,
        }
    }

    /// Depth from the centre. Lower may not depend on higher.
    fn depth(self) -> u8 {
        match self {
            Layer::Domain => 0,
            Layer::Application => 1,
            Layer::Adapters => 2,
            Layer::Composition => 3,
            Layer::Tooling => 4,
        }
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Layer::Domain => "Domain",
            Layer::Application => "Application",
            Layer::Adapters => "Adapters",
            Layer::Composition => "Composition",
            Layer::Tooling => "Tooling",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerEdge {
    pub from: String,
    pub to: String,
    pub from_layer: Layer,
    pub to_layer: Layer,
    /// Manifest file and line that declares the dependency, so a reviewer can
    /// find it without searching.
    pub declared_at: String,
}

/// Decides whether an edge is permitted. Inward and same-layer are allowed;
/// outward is not.
pub struct LayerRule;

impl LayerRule {
    pub fn is_permitted(edge: &LayerEdge) -> bool {
        edge.to_layer.depth() <= edge.from_layer.depth()
    }

    pub fn describe_violation(edge: &LayerEdge) -> String {
        format!(
            "{} ({}) depends on {} ({}); dependencies point inward only [{}]",
            edge.from, edge.from_layer, edge.to, edge.to_layer, edge.declared_at
        )
    }
}

/// Languages for which a boundary rule exists.
///
/// FR-003: the gate refuses source it has no rule for. Passing would report
/// that boundaries hold across the product when a whole language went
/// unexamined, and a gate that reports more than it checked is worse than one
/// that refuses, because it is believed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
}

impl Language {
    /// The file extensions the gate can judge.
    pub const fn extensions() -> &'static [&'static str] {
        &["rs"]
    }

    /// Extensions that carry no layering: manifests, data, documentation, and
    /// shell scripts, which have no imports and so no direction to enforce.
    ///
    /// This list is the gate's blind spot, so it stays short and each entry
    /// earns its place. Adding a real language here instead of writing a rule
    /// for it would defeat the refusal above.
    pub const fn ignored() -> &'static [&'static str] {
        &[
            "toml", "md", "json", "lock", "txt", "png", "svg", "woff2", "ttf", "css", "html",
            "sh",
        ]
    }

    pub fn of_extension(extension: &str) -> Option<Self> {
        matches!(extension, "rs").then_some(Language::Rust)
    }
}
