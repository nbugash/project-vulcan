//! Reads declared layers and dependency edges from the Cargo workspace.
//!
//! Most layering violations are already compile errors, because a crate cannot
//! import what it does not declare. This adapter supplies the edge the compiler
//! accepts: a declared dependency pointing the wrong way.

use std::path::{Path, PathBuf};

use vulcan_app::ports::workspace_graph::{GraphError, WorkspaceGraphPort};
use vulcan_domain::layer_edge::{Layer, LayerEdge};

pub struct CargoManifestAdapter;

struct CrateManifest {
    name: String,
    layer: Layer,
    path: PathBuf,
    dependencies: Vec<(String, usize)>,
}

/// Where product source lives. Governance tooling under `.specify/` is
/// deliberately outside: it is not the product, it declares no layer, and
/// including it would make the gate refuse on every run for Python that has no
/// layering to check.
const GOVERNED_ROOTS: [&str; 2] = ["crates", "tools"];

impl WorkspaceGraphPort for CargoManifestAdapter {
    fn source_extensions(&self, manifest_path: &Path) -> Result<Vec<String>, GraphError> {
        let mut found = Vec::new();
        for root in GOVERNED_ROOTS {
            collect_extensions(&manifest_path.join(root), &mut found)?;
        }
        found.sort();
        found.dedup();
        Ok(found)
    }

    fn read_edges(&self, manifest_path: &Path) -> Result<Vec<LayerEdge>, GraphError> {
        let root = manifest_path.join("Cargo.toml");
        let text = std::fs::read_to_string(&root).map_err(|error| GraphError::Unparseable {
            path: root.display().to_string(),
            detail: error.to_string(),
        })?;
        let document: toml::Value = toml::from_str(&text).map_err(|error| GraphError::Unparseable {
            path: root.display().to_string(),
            detail: error.to_string(),
        })?;

        let members = document
            .get("workspace")
            .and_then(|workspace| workspace.get("members"))
            .and_then(|members| members.as_array())
            .ok_or_else(|| GraphError::Unparseable {
                path: root.display().to_string(),
                detail: "no [workspace] members".into(),
            })?;

        let mut manifests = Vec::new();
        for member in members {
            let relative = member.as_str().unwrap_or_default();
            manifests.push(read_member(manifest_path.join(relative))?);
        }

        let layer_of = |name: &str| manifests.iter().find(|m| m.name == name).map(|m| m.layer);

        let mut edges = Vec::new();
        for manifest in &manifests {
            for (dependency, line) in &manifest.dependencies {
                // Only workspace-internal edges carry layering meaning; a
                // third-party crate has no layer to compare against.
                if let Some(to_layer) = layer_of(dependency) {
                    edges.push(LayerEdge {
                        from: manifest.name.clone(),
                        to: dependency.clone(),
                        from_layer: manifest.layer,
                        to_layer,
                        declared_at: format!("{}:{}", manifest.path.display(), line),
                    });
                }
            }
        }
        Ok(edges)
    }
}

fn read_member(directory: PathBuf) -> Result<CrateManifest, GraphError> {
    let path = directory.join("Cargo.toml");
    let text = std::fs::read_to_string(&path).map_err(|error| GraphError::Unparseable {
        path: path.display().to_string(),
        detail: error.to_string(),
    })?;
    let document: toml::Value = toml::from_str(&text).map_err(|error| GraphError::Unparseable {
        path: path.display().to_string(),
        detail: error.to_string(),
    })?;

    let name = document
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(|name| name.as_str())
        .unwrap_or_default()
        .to_string();

    let declared = document
        .get("package")
        .and_then(|package| package.get("metadata"))
        .and_then(|metadata| metadata.get("vulcan"))
        .and_then(|vulcan| vulcan.get("layer"))
        .and_then(|layer| layer.as_str());

    let layer = declared.and_then(Layer::parse).ok_or_else(|| GraphError::UndeclaredLayer {
        crate_name: if name.is_empty() { directory.display().to_string() } else { name.clone() },
        path: path.display().to_string(),
    })?;

    let dependencies = document
        .get("dependencies")
        .and_then(|dependencies| dependencies.as_table())
        .map(|table| {
            table
                .keys()
                .map(|key| (key.clone(), line_of(&text, key)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(CrateManifest { name, layer, path, dependencies })
}

/// The manifest line that declares a dependency, so a finding points at
/// something a reviewer can open rather than at a file.
fn line_of(text: &str, key: &str) -> usize {
    text.lines()
        .position(|line| line.trim_start().starts_with(key))
        .map(|index| index + 1)
        .unwrap_or(0)
}

/// Skips build output: `target/` holds generated source in whatever language the
/// toolchain emits, and none of it is the product's.
fn collect_extensions(dir: &Path, found: &mut Vec<String>) -> Result<(), GraphError> {
    if !dir.exists() || dir.file_name().is_some_and(|name| name == "target") {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir).map_err(|error| GraphError::Unparseable {
        path: dir.display().to_string(),
        detail: error.to_string(),
    })?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_extensions(&path, found)?;
        } else if let Some(extension) = path.extension() {
            found.push(extension.to_string_lossy().to_lowercase());
        }
    }
    Ok(())
}
