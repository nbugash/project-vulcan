//! Gate 1: architectural boundaries.
//!
//! Exit 0 judged and passed, 2 judged and failed, 1 could not judge.

use std::path::PathBuf;

use vulcan_adapters::manifest::cargo_manifest::CargoManifestAdapter;
use vulcan_app::use_cases::check_boundaries::CheckBoundaries;
use vulcan_cli::gate_entry::{finish, Args};

fn main() {
    let args = Args::parse(std::env::args().skip(1));
    let manifest_path = PathBuf::from(args.flag("manifest-path").unwrap_or("."));

    let outcome = CheckBoundaries::new(CargoManifestAdapter).execute(&manifest_path);
    let context = vec![("manifest_path".into(), manifest_path.display().to_string())];

    std::process::exit(finish("gate-boundary", &args, outcome, context));
}
