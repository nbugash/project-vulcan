//! Gate 8, lint: reject design values the prototype does not define.

use vulcan_domain::design_value::TokenSet;
use vulcan_domain::verdict::GateVerdict;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    pub value: String,
    pub file: String,
    pub line: usize,
}

pub struct LintOffToken;

impl LintOffToken {
    /// A literal passes if the design system names it, or if the prototype uses
    /// it unnamed. It fails only when the reproduction invented it.
    pub fn execute(tokens: &TokenSet, vocabulary: &[String], found: &[Literal]) -> GateVerdict {
        let findings: Vec<String> = found
            .iter()
            .filter(|literal| {
                !tokens.contains_literal(&literal.value)
                    && !vocabulary.iter().any(|known| known == &literal.value)
            })
            .map(|literal| {
                format!(
                    "{} appears nowhere in the prototype [{}:{}]",
                    literal.value, literal.file, literal.line
                )
            })
            .collect();

        if findings.is_empty() {
            GateVerdict::Passed
        } else {
            GateVerdict::Failed { findings }
        }
    }
}
