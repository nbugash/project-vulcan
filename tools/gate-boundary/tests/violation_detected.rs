//! T031: the quickstart scenario, end to end.
//!
//! A gate that has never been seen to fail has not been validated, so each case
//! here asserts a distinct exit code rather than only the happy path.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

/// A two-crate workspace: a domain crate and an adapters crate.
fn workspace(root: &Path, domain_deps: &str) {
    write(
        &root.join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = [\"crates/demo-domain\", \"crates/demo-adapters\"]\n",
    );
    write(
        &root.join("crates/demo-domain/Cargo.toml"),
        &format!(
            "[package]\nname = \"demo-domain\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
             [package.metadata.vulcan]\nlayer = \"Domain\"\n\n[dependencies]\n{domain_deps}"
        ),
    );
    write(
        &root.join("crates/demo-adapters/Cargo.toml"),
        "[package]\nname = \"demo-adapters\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
         [package.metadata.vulcan]\nlayer = \"Adapters\"\n\n[dependencies]\n\
         demo-domain = { path = \"../demo-domain\" }\n",
    );
}

fn run(root: &Path) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_gate-boundary"))
        .args(["--manifest-path", root.to_str().unwrap()])
        .output()
        .expect("gate-boundary runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
    )
}

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("vulcan-gate-boundary-{name}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn a_clean_workspace_passes() {
    let root = scratch("clean");
    workspace(&root, "");
    let (code, output) = run(&root);
    assert_eq!(code, 0, "{output}");
    assert!(output.contains("PASS"));
}

#[test]
fn a_reversed_dependency_fails_and_names_the_manifest_line() {
    let root = scratch("violation");
    workspace(&root, "demo-adapters = { path = \"../demo-adapters\" }\n");
    let (code, output) = run(&root);
    assert_eq!(code, 2, "{output}");
    assert!(output.contains("demo-domain"));
    assert!(output.contains("demo-adapters"));
    assert!(output.contains("Cargo.toml:"), "finding should carry a manifest location: {output}");
}

#[test]
fn removing_the_violation_restores_a_pass() {
    let root = scratch("restored");
    workspace(&root, "demo-adapters = { path = \"../demo-adapters\" }\n");
    assert_eq!(run(&root).0, 2);
    workspace(&root, "");
    assert_eq!(run(&root).0, 0);
}

#[test]
fn a_crate_with_no_declared_layer_cannot_be_judged() {
    let root = scratch("undeclared");
    workspace(&root, "");
    write(
        &root.join("crates/demo-domain/Cargo.toml"),
        "[package]\nname = \"demo-domain\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
    );
    let (code, output) = run(&root);
    // Exit 1, not 2: a gate that could not judge has not failed, and must never
    // be mistaken for one that passed.
    assert_eq!(code, 1, "{output}");
    assert!(output.contains("COULD NOT JUDGE"));
}
