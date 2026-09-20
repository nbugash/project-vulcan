//! Gate 8: prototype fidelity.

use std::path::{Path, PathBuf};

use vulcan_adapters::rendering::exact_compare::ExactImageComparator;
use vulcan_adapters::rendering::gpui_capture::GpuiCaptureAdapter;
use vulcan_adapters::rendering::reference_store::{ReferenceMetadata, ReferenceStore};
use vulcan_adapters::rendering::visual_report::write_difference;
use vulcan_adapters::tokens::stylesheet::{discover_prototype, StylesheetAndManifestAdapter};
use vulcan_app::ports::render_capture::{RenderCapturePort, RenderError};
use vulcan_app::use_cases::compare_fidelity::{
    verdict_of, CompareFidelity, CompareInput, Outcome, ReferenceDigests,
};
use vulcan_domain::rendering::Viewport;
use vulcan_adapters::reporting::discrepancies::{read_all, restatements, Status};
use vulcan_adapters::tokens::scanner::SourceScanner;
use vulcan_app::ports::token_source::TokenSourcePort;
use vulcan_app::use_cases::extract_tokens::ExtractTokens;
use vulcan_app::use_cases::lint_off_token::LintOffToken;
use vulcan_cli::gate_entry::{finish, Args};
use vulcan_domain::verdict::{GateError, GateVerdict};

fn main() {
    let args = Args::parse(std::env::args().skip(1));
    let subcommand = args.positional.first().cloned().unwrap_or_else(|| "extract".into());
    let mockups = PathBuf::from(args.flag("prototype").unwrap_or("mockups"));

    let outcome: Result<GateVerdict, GateError> = match subcommand.as_str() {
        "extract" => match StylesheetAndManifestAdapter::discover(&mockups) {
            Ok(adapter) => match ExtractTokens::new(adapter).execute() {
                Ok(tokens) => {
                    let path = PathBuf::from("tools/gate-fidelity/out/tokens.json");
                    let module = PathBuf::from("crates/vulcan-ui/src/generated_tokens.rs");
                    match write_tokens(&path, &tokens).and_then(|_| write_module(&module, &tokens)) {
                        Err(error) => Err(GateError::CouldNotJudge(format!("cannot write tokens: {error}"))),
                        Ok(()) => {
                            println!(
                                "extracted {} design values to {} and {}",
                                tokens.len(),
                                path.display(),
                                module.display()
                            );
                            Ok(GateVerdict::Passed)
                        }
                    }
                }
                Err(error) => Err(error),
            },
            Err(error) => Err(GateError::CouldNotJudge(format!("{error:?}"))),
        },
        "lint" => lint(&mockups, &args),
        "discrepancies" => discrepancies(&args),
        "capture-reference" => capture_reference(&mockups),
        "compare" => compare(&mockups),
        other => Err(GateError::CouldNotJudge(format!("unknown subcommand {other}"))),
    };

    std::process::exit(finish("gate-fidelity", &args, outcome, vec![("subcommand".into(), subcommand)]));
}

/// Reads the prototype's design values, then reads the reproduction's source,
/// and reports every value in the second that is not in the first.
fn lint(mockups: &Path, args: &Args) -> Result<GateVerdict, GateError> {
    let adapter = StylesheetAndManifestAdapter::discover(mockups)
        .map_err(|error| GateError::CouldNotJudge(format!("{error:?}")))?;
    let vocabulary = adapter.read_vocabulary().map_err(|error| {
        GateError::CouldNotJudge(format!("cannot read the prototype's vocabulary: {error:?}"))
    })?;
    let tokens = ExtractTokens::new(adapter).execute()?;

    let roots = args.flag("source").unwrap_or(LINT_SOURCE);
    let found = SourceScanner::new(roots.split(',').map(PathBuf::from))
        .scan()
        .map_err(|error| GateError::CouldNotJudge(format!("cannot read source: {error}")))?;

    println!("scanned {} design literals in {roots}", found.len());
    Ok(LintOffToken::execute(&tokens, &vocabulary, &found))
}

/// Gate 8, triage. An untriaged discrepancy is a decision nobody has taken,
/// and shipping one silently converts it into an accepted difference.
fn discrepancies(args: &Args) -> Result<GateVerdict, GateError> {
    let dir = PathBuf::from(args.flag("discrepancies").unwrap_or(DISCREPANCY_DIR));
    let found = read_all(&dir)
        .map_err(|error| GateError::CouldNotJudge(format!("cannot read {}: {error}", dir.display())))?;

    let map = PathBuf::from(args.flag("map").unwrap_or(FEATURE_MAP));
    let backlog = std::fs::read_to_string(&map)
        .map_err(|error| GateError::CouldNotJudge(format!("cannot read {}: {error}", map.display())))?;

    let mut findings = Vec::new();
    for discrepancy in &found {
        match discrepancy.status {
            Status::Untriaged => findings.push(format!(
                "{} is untriaged; decide it as Accepted or BacklogEntry before review",
                discrepancy.path
            )),
            Status::Unreadable => findings.push(format!(
                "{} has no decided Status line",
                discrepancy.path
            )),
            Status::BacklogEntry => match &discrepancy.backlog_entry {
                None => findings.push(format!(
                    "{} is a BacklogEntry but names no feature",
                    discrepancy.path
                )),
                Some(entry) if !backlog.contains(entry.as_str()) => findings.push(format!(
                    "{} names backlog entry {entry}, which is not in {}",
                    discrepancy.path,
                    map.display()
                )),
                Some(_) => {}
            },
            Status::Accepted => {}
        }
    }

    let design = PathBuf::from(args.flag("designs").unwrap_or(DESIGN_DIR));
    findings.extend(
        restatements(&design, &found)
            .map_err(|error| GateError::CouldNotJudge(format!("cannot read {}: {error}", design.display())))?,
    );

    println!("triaged {} discrepancies in {}", found.len(), dir.display());
    if findings.is_empty() {
        Ok(GateVerdict::Passed)
    } else {
        Ok(GateVerdict::Failed { findings })
    }
}

const DISCREPANCY_DIR: &str = "reports/mockups-discrepancies";
const DESIGN_DIR: &str = "docs/system-designs";
const FEATURE_MAP: &str = "specs/features-map.md";
const LINT_SOURCE: &str = "crates/vulcan-ui/src";
const REFERENCE_DIR: &str = "tools/gate-fidelity/reference";
const COMPOSITION: &str = "shell";
const VIEWPORT: Viewport = Viewport { width: 1440, height: 900 };

/// A digest of the prototype and one of the environment, so a later comparison
/// failure can name its cause rather than only its size.
fn digests(mockups: &Path) -> (String, String) {
    let prototype = discover_prototype(mockups)
        .ok()
        .and_then(|path| std::fs::read(path).ok())
        .map(|bytes| format!("fnv1a:{:016x}", fnv1a(&bytes)))
        .unwrap_or_else(|| "absent".into());
    let environment = std::env::var("VULCAN_PINNED_ENV_VERSION").unwrap_or_else(|_| "unpinned".into());
    (prototype, environment)
}

/// FNV-1a over the prototype's bytes.
///
/// A length was not a digest: any edit that preserves the byte count — a colour
/// swapped for another of the same width, two attributes reordered — left the
/// reference looking current. This is not cryptographic and does not need to be;
/// it detects change, and it is written out rather than taken from the standard
/// library because `DefaultHasher`'s output is not guaranteed stable across
/// Rust versions, and a stored digest has to reproduce.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn capture_reference(mockups: &Path) -> Result<GateVerdict, GateError> {
    let image = GpuiCaptureAdapter::default().capture(VIEWPORT).map_err(|error| {
        GateError::CouldNotJudge(match error {
            RenderError::NotPinnedEnvironment => "capture-reference must run inside the pinned environment".into(),
            RenderError::TypefaceMissing(font) => format!("typeface missing: {font}; a reference captured now would record a substitute"),
            RenderError::Failed(detail) => detail,
        })
    })?;

    let (prototype, environment) = digests(mockups);
    let metadata = ReferenceMetadata {
        viewport: VIEWPORT,
        typefaces: vec!["Inter 400/500/600".into(), "Phosphor 2.1.1".into()],
        prototype_digest: prototype,
        environment_digest: environment,
        captured_at: "captured".into(),
    };

    ReferenceStore::new(REFERENCE_DIR)
        .save(COMPOSITION, &image, &metadata)
        .map_err(|error| GateError::CouldNotJudge(format!("{error:?}")))?;

    println!("reference captured to {REFERENCE_DIR}/{COMPOSITION}.png");
    Ok(GateVerdict::Passed)
}

fn compare(mockups: &Path) -> Result<GateVerdict, GateError> {
    // Before anything that could produce a verdict. A stale reference or a
    // missing one is a finding only if this process was entitled to judge at
    // all, and outside the pinned environment it is not.
    GpuiCaptureAdapter::in_pinned_environment()
        .map_err(|_| GateError::CouldNotJudge("compare must run inside the pinned environment".into()))?;

    let store = ReferenceStore::new(REFERENCE_DIR);
    let (reference, metadata) = store
        .load(COMPOSITION)
        .map_err(|_| GateError::CouldNotJudge(
            "no reference captured yet; run capture-reference inside the pinned environment".into(),
        ))?;

    let (prototype, environment) = digests(mockups);
    let use_case = CompareFidelity::new(GpuiCaptureAdapter::default(), ExactImageComparator);
    let outcome = use_case.execute(CompareInput {
        viewport: VIEWPORT,
        reference,
        recorded: ReferenceDigests {
            prototype: metadata.prototype_digest,
            environment: metadata.environment_digest,
        },
        current: ReferenceDigests { prototype, environment },
    })?;

    // A failed comparison leaves an image, not only a number.
    if let Outcome::Differs { actual, .. } = &outcome {
        let report = PathBuf::from(REFERENCE_DIR).join("difference.png");
        if let Ok((stored, _)) = ReferenceStore::new(REFERENCE_DIR).load(COMPOSITION) {
            if let Ok(marked) = write_difference(&report, actual, &stored) {
                println!("{marked} pixels marked in {}", report.display());
            }
        }
    }

    Ok(verdict_of(&outcome))
}

/// Emits the token set as Rust constants. Generated rather than transcribed: a
/// transcription is a fork of the design system that diverges on the first
/// prototype change, where a regenerated module shows up as a diff.
fn write_module(path: &PathBuf, tokens: &vulcan_domain::design_value::TokenSet) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut out = String::from(
        "//! Design values extracted from the signed-off prototype.\n//!\n         //! GENERATED by `gate-fidelity extract`. Do not edit: a hand edit is a fork of\n         //! the design system, and the off-token lint will reject any value that is not\n         //! here.\n\n",
    );
    for value in tokens.values() {
        let ident = value.name.replace('-', "_").to_uppercase();
        out.push_str(&format!(
            "/// `{}` from {}\npub const {}: &str = {};\n\n",
            value.name,
            value.source_location,
            ident,
            quote(&value.value)
        ));
    }
    std::fs::write(path, out)
}

/// Token values carry quotes of their own, such as a font stack, so every
/// string is escaped rather than interpolated.
fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn write_tokens(path: &PathBuf, tokens: &vulcan_domain::design_value::TokenSet) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let rows: Vec<String> = tokens
        .values()
        .iter()
        .map(|value| {
            format!(
                "    {{ \"name\": {}, \"value\": {}, \"kind\": \"{:?}\", \"source\": \"{:?}\", \"source_location\": {} }}",
                quote(&value.name),
                quote(&value.value),
                value.kind,
                value.source,
                quote(&value.source_location)
            )
        })
        .collect();
    std::fs::write(path, format!("{{\n  \"values\": [\n{}\n  ]\n}}", rows.join(",\n")))
}
