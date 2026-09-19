//! Design values extracted from the signed-off prototype.
//!
//! The only permitted source of appearance in the product. Extraction reads both
//! the stylesheet's custom properties and the values documented only in the
//! asset manifest, because the design system is monochrome and the state and
//! syntax colours were added outside it.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Colour,
    Typeface,
    Space,
    Radius,
    Shadow,
    Density,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    Stylesheet,
    Manifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignValue {
    pub name: String,
    pub value: String,
    pub kind: ValueKind,
    pub source: ValueSource,
    pub source_location: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenSetError {
    /// The two sources disagree, so the appearance contract is ambiguous and
    /// must be fixed at the source rather than merged here.
    DuplicateName { name: String, first: String, second: String },
    InvalidName(String),
}

/// The extracted set, sorted by name so two runs over an unchanged prototype
/// produce byte-identical output.
#[derive(Debug, Clone, Default)]
pub struct TokenSet {
    values: Vec<DesignValue>,
}

impl TokenSet {
    pub fn build(mut values: Vec<DesignValue>) -> Result<Self, TokenSetError> {
        for value in &values {
            if !Self::is_valid_name(&value.name) {
                return Err(TokenSetError::InvalidName(value.name.clone()));
            }
        }
        values.sort_by(|a, b| a.name.cmp(&b.name));
        for pair in values.windows(2) {
            if pair[0].name == pair[1].name {
                return Err(TokenSetError::DuplicateName {
                    name: pair[0].name.clone(),
                    first: pair[0].source_location.clone(),
                    second: pair[1].source_location.clone(),
                });
            }
        }
        Ok(Self { values })
    }

    pub fn values(&self) -> &[DesignValue] {
        &self.values
    }

    pub fn contains_literal(&self, literal: &str) -> bool {
        self.values.iter().any(|value| value.value == literal)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn is_valid_name(name: &str) -> bool {
        let mut chars = name.chars();
        match chars.next() {
            Some(first) if first.is_ascii_lowercase() || first.is_ascii_digit() => {}
            _ => return false,
        }
        name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }
}
