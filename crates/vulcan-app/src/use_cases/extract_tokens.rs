//! Gate 8, extraction: produce the design value set the product may use.
//!
//! Deterministic by construction: the set is sorted and duplicate names are an
//! error rather than a merge, so two runs over an unchanged prototype produce
//! byte-identical output and a diff means the prototype moved.

use vulcan_domain::design_value::{TokenSet, TokenSetError};
use vulcan_domain::verdict::GateError;

use crate::ports::token_source::{TokenError, TokenSourcePort};

pub struct ExtractTokens<S: TokenSourcePort> {
    source: S,
}

impl<S: TokenSourcePort> ExtractTokens<S> {
    pub fn new(source: S) -> Self {
        Self { source }
    }

    pub fn execute(&self) -> Result<TokenSet, GateError> {
        let values = self.source.read_values().map_err(|error| {
            GateError::CouldNotJudge(match error {
                TokenError::Unparseable { path, detail } => format!("cannot parse {path}: {detail}"),
                TokenError::MissingSource(detail) => format!("missing token source: {detail}"),
            })
        })?;

        TokenSet::build(values).map_err(|error| {
            GateError::CouldNotJudge(match error {
                TokenSetError::DuplicateName { name, first, second } => format!(
                    "{name} is declared in both {first} and {second}; the appearance contract is ambiguous and must be fixed at the source"
                ),
                TokenSetError::InvalidName(name) => format!("{name} is not a valid token name"),
            })
        })
    }
}
