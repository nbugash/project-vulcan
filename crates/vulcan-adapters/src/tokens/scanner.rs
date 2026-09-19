//! Gate 8, lint: finding design literals in the reproduction's own source.
//!
//! The visual comparison cannot catch this class. Its reference is captured
//! *from* the shell, so a colour hardcoded before the capture is baked into the
//! reference and compares clean forever. Only reading the source catches a value
//! that never came from the prototype.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use vulcan_app::use_cases::lint_off_token::Literal;

/// Source files that define design values rather than consume them, and so
/// cannot be linted against themselves.
const GENERATED: &str = "generated_tokens.rs";

pub struct SourceScanner {
    roots: Vec<PathBuf>,
}

impl SourceScanner {
    pub fn new(roots: impl IntoIterator<Item = PathBuf>) -> Self {
        Self { roots: roots.into_iter().collect() }
    }

    pub fn scan(&self) -> io::Result<Vec<Literal>> {
        let mut found = Vec::new();
        for root in &self.roots {
            collect(root, &mut found)?;
        }
        // Stable order: a lint that reports findings in directory order gives a
        // different diff on every machine.
        found.sort_by(|a, b| (&a.file, a.line, &a.value).cmp(&(&b.file, b.line, &b.value)));
        Ok(found)
    }
}

fn collect(path: &Path, found: &mut Vec<Literal>) -> io::Result<()> {
    if path.is_dir() {
        let mut entries: Vec<PathBuf> =
            fs::read_dir(path)?.map(|entry| entry.map(|e| e.path())).collect::<Result<_, _>>()?;
        entries.sort();
        for entry in entries {
            collect(&entry, found)?;
        }
        return Ok(());
    }

    if path.extension().is_none_or(|ext| ext != "rs")
        || path.file_name().is_some_and(|name| name == GENERATED)
    {
        return Ok(());
    }

    let text = fs::read_to_string(path)?;
    let file = path.to_string_lossy().replace('\\', "/");
    for (index, line) in text.lines().enumerate() {
        for value in literals_in(strip_comment(line)) {
            found.push(Literal { value, file: file.clone(), line: index + 1 });
        }
    }
    Ok(())
}

/// Drops a trailing line comment. A comment naming a colour is documentation,
/// not a design decision the renderer acts on.
fn strip_comment(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

/// Design literals as the renderer expresses them: `rgb(0x1e1f22)` is a colour,
/// `px(8.0)` is a dimension. Both are normalised to the form the extracted set
/// uses, so comparison is a string match rather than a unit conversion.
fn literals_in(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (call, normalise) in
        [("rgb(0x", colour as fn(&str) -> Option<String>), ("rgba(0x", colour), ("px(", dimension)]
    {
        let mut rest = line;
        while let Some(at) = rest.find(call) {
            let after = &rest[at + call.len()..];
            let end = after.find(')').unwrap_or(after.len());
            if let Some(value) = normalise(&after[..end]) {
                found.push(value);
            }
            rest = &after[end.min(after.len())..];
        }
    }
    found
}

fn colour(raw: &str) -> Option<String> {
    let hex: String = raw.trim().chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    (hex.len() == 6).then(|| format!("#{}", hex.to_lowercase()))
}

fn dimension(raw: &str) -> Option<String> {
    let number: f64 = raw.trim().parse().ok()?;
    // `px(8.0)` and the token `"8px"` are the same value written two ways.
    let text = if number.fract() == 0.0 {
        format!("{}", number as i64)
    } else {
        format!("{number}")
    };
    Some(format!("{text}px"))
}
