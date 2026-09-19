//! Reads design values from the prototype's two sources.
//!
//! The stylesheet carries the custom properties. The asset manifest carries the
//! values that are not custom properties: the design system is deliberately
//! monochrome, so the state and syntax colours were added outside it. Reading
//! only the stylesheet would miss them, and the off-token lint would then reject
//! them as violations the moment anyone used them correctly.

use std::path::{Path, PathBuf};

use vulcan_app::ports::token_source::{TokenError, TokenSourcePort};
use vulcan_domain::design_value::{DesignValue, ValueKind, ValueSource};

pub struct StylesheetAndManifestAdapter {
    stylesheet: PathBuf,
    manifest: PathBuf,
    prototype: PathBuf,
}

impl StylesheetAndManifestAdapter {
    pub fn new(
        stylesheet: impl Into<PathBuf>,
        manifest: impl Into<PathBuf>,
        prototype: impl Into<PathBuf>,
    ) -> Self {
        Self {
            stylesheet: stylesheet.into(),
            manifest: manifest.into(),
            prototype: prototype.into(),
        }
    }

    /// Locates the design system stylesheet, whose directory name carries a
    /// generated identifier.
    pub fn discover(mockups: &Path) -> Result<Self, TokenError> {
        let design_system = mockups.join("_ds");
        let entry = std::fs::read_dir(&design_system)
            .map_err(|error| TokenError::MissingSource(format!("{}: {error}", design_system.display())))?
            .filter_map(Result::ok)
            .find(|entry| entry.path().join("styles.css").exists())
            .ok_or_else(|| TokenError::MissingSource("no _ds/*/styles.css".into()))?;

        Ok(Self::new(
            entry.path().join("styles.css"),
            mockups.join("assets.md"),
            discover_prototype(mockups)?,
        ))
    }
}

/// Locates the signed-off prototype without depending on its filename.
///
/// It was `Vulcan-IDE.html` and became `Vulcan IDE (standalone).html`; a
/// hardcoded name turned that rename into a gate that read no prototype at all.
/// The criterion here is a property the prototype must have anyway: it is
/// self-contained, referencing no sibling asset directory and no remote origin,
/// because a reference render that depends on files outside itself is not
/// reproducible.
pub fn discover_prototype(mockups: &Path) -> Result<PathBuf, TokenError> {
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(mockups)
        .map_err(|error| TokenError::MissingSource(format!("{}: {error}", mockups.display())))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "html"))
        .collect();
    candidates.sort();

    if candidates.is_empty() {
        return Err(TokenError::MissingSource(format!(
            "no prototype: {} contains no .html",
            mockups.display()
        )));
    }

    let self_contained: Vec<PathBuf> = candidates
        .iter()
        .filter(|path| {
            std::fs::read_to_string(path).map(|text| vulcan_domain::prototype::is_self_contained(&text)).unwrap_or(false)
        })
        .cloned()
        .collect();

    match self_contained.as_slice() {
        [only] => Ok(only.clone()),
        [] => Err(TokenError::MissingSource(format!(
            "no self-contained prototype among {:?}; each references assets outside itself",
            names(&candidates)
        ))),
        many => Err(TokenError::MissingSource(format!(
            "ambiguous prototype: {:?} are all self-contained; name one with --prototype",
            names(many)
        ))),
    }
}

fn names(paths: &[PathBuf]) -> Vec<String> {
    paths.iter().map(|path| path.file_name().unwrap_or_default().to_string_lossy().into_owned()).collect()
}

impl TokenSourcePort for StylesheetAndManifestAdapter {
    fn read_values(&self) -> Result<Vec<DesignValue>, TokenError> {
        let mut values = read_custom_properties(&self.stylesheet)?;
        values.extend(read_manifest_values(&self.manifest)?);
        Ok(values)
    }

    fn read_vocabulary(&self) -> Result<Vec<String>, TokenError> {
        let mut vocabulary = Vec::new();
        for source in [&self.stylesheet, &self.prototype] {
            let text = std::fs::read_to_string(source).map_err(|error| {
                TokenError::MissingSource(format!("{}: {error}", source.display()))
            })?;
            vocabulary.extend(pixel_lengths(&text));
        }
        vocabulary.sort();
        vocabulary.dedup();
        Ok(vocabulary)
    }
}

/// Every `<number>px` in the text. Deliberately blunt: the question is whether
/// a value occurs in the signed-off artefact, not where in its grammar it sits.
fn pixel_lengths(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(offset) = text[at..].find("px") {
        let end = at + offset;
        let mut start = end;
        while start > 0 && (bytes[start - 1].is_ascii_digit() || bytes[start - 1] == b'.') {
            start -= 1;
        }
        let number = text[start..end].trim_matches('.');
        if !number.is_empty() && number.parse::<f64>().is_ok() {
            found.push(format!("{number}px"));
        }
        at = end + 2;
    }
    found
}

fn read_custom_properties(path: &Path) -> Result<Vec<DesignValue>, TokenError> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| TokenError::MissingSource(format!("{}: {error}", path.display())))?;

    let mut values = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("--") else { continue };
        let Some((name, raw)) = rest.split_once(':') else { continue };
        // The stylesheet carries design-review annotations beside values, so the
        // comment is stripped before the semicolon rather than after it.
        let without_comment = raw.split("/*").next().unwrap_or(raw);
        let trimmed = without_comment.trim().trim_end_matches(';').trim();
        // A font stack opens with a quoted family and continues unquoted, so
        // quotes are stripped only when they wrap the whole value.
        let value = match (trimmed.starts_with('"'), trimmed.ends_with('"'), trimmed.len() > 1) {
            (true, true, true) => trimmed[1..trimmed.len() - 1].to_string(),
            _ => trimmed.to_string(),
        };
        if value.is_empty() {
            continue;
        }
        values.push(DesignValue {
            name: name.trim().to_string(),
            value,
            kind: classify(name.trim()),
            source: ValueSource::Stylesheet,
            source_location: format!("{}:{}", path.display(), index + 1),
        });
    }
    Ok(values)
}

/// Colours documented in prose tables rather than as custom properties. They are
/// design values despite not being tokens in the stylesheet.
fn read_manifest_values(path: &Path) -> Result<Vec<DesignValue>, TokenError> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| TokenError::MissingSource(format!("{}: {error}", path.display())))?;

    let mut values = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();

        // Fixed chrome heights are declared in prose rather than as custom
        // properties, and FR-029 requires regions use exactly these.
        if trimmed.starts_with("**Fixed chrome heights**") {
            for part in trimmed.split('·') {
                if let Some((name, extent)) = parse_extent(part) {
                    values.push(DesignValue {
                        name: format!("chrome-{name}"),
                        value: extent,
                        kind: ValueKind::Space,
                        source: ValueSource::Manifest,
                        source_location: format!("{}:{}", path.display(), index + 1),
                    });
                }
            }
            continue;
        }

        // Density table: | `--vk-line` | 19px | 21px | 25px |
        if trimmed.starts_with("| `--vk-") {
            let cells: Vec<&str> = trimmed.split('|').map(str::trim).collect();
            if cells.len() >= 6 {
                // The cell carries a description after the token: `--vk-line` (code line height)
                let Some(base) = between(cells[1], "`--", "`") else { continue };
                for (profile, cell) in [("compact", cells[2]), ("default", cells[3]), ("roomy", cells[4])] {
                    if !cell.is_empty() {
                        values.push(DesignValue {
                            name: format!("{base}-{profile}"),
                            value: cell.to_string(),
                            kind: ValueKind::Density,
                            source: ValueSource::Manifest,
                            source_location: format!("{}:{}", path.display(), index + 1),
                        });
                    }
                }
            }
            continue;
        }

        if !trimmed.starts_with("| `#") {
            continue;
        }
        let Some(hex) = between(trimmed, "`#", "`") else { continue };
        let hex = format!("#{hex}");
        if !seen.insert(hex.clone()) {
            continue;
        }
        let role = trimmed.split('|').nth(2).unwrap_or_default().trim();
        values.push(DesignValue {
            name: slug(&hex, role),
            value: hex,
            kind: ValueKind::Colour,
            source: ValueSource::Manifest,
            source_location: format!("{}:{}", path.display(), index + 1),
        });
    }
    Ok(values)
}

/// "toolbar 46px" -> ("toolbar", "46px"); "rail width 44px" -> ("rail-width", "44px").
fn parse_extent(part: &str) -> Option<(String, String)> {
    let cleaned = part
        .trim()
        .trim_start_matches("**Fixed chrome heights**")
        .trim()
        .trim_start_matches('—')
        .trim();
    let (name, extent) = cleaned.rsplit_once(' ')?;
    let extent = extent.trim().trim_end_matches('.');
    if !extent.ends_with("px") {
        return None;
    }
    let slug = name
        .trim()
        .split_whitespace()
        .map(|word| word.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("-");
    (!slug.is_empty()).then(|| (slug, extent.to_string()))
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let rest = &text[from..];
    let to = rest.find(end)?;
    Some(&rest[..to])
}

/// Manifest colours have a role rather than a token name, so one is derived from
/// the role and kept stable by including the value.
fn slug(hex: &str, role: &str) -> String {
    let words: String = role
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .take(3)
        .map(|word| word.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("-");
    let suffix = hex.trim_start_matches('#').to_ascii_lowercase();
    if words.is_empty() {
        format!("manifest-{suffix}")
    } else {
        format!("manifest-{words}-{suffix}")
    }
}

fn classify(name: &str) -> ValueKind {
    if name.starts_with("color") {
        ValueKind::Colour
    } else if name.starts_with("font") {
        ValueKind::Typeface
    } else if name.starts_with("space") {
        ValueKind::Space
    } else if name.starts_with("radius") {
        ValueKind::Radius
    } else if name.starts_with("shadow") {
        ValueKind::Shadow
    } else {
        ValueKind::Density
    }
}
