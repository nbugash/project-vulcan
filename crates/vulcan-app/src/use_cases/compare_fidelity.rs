//! Gate 8, comparison: judge a rendering against the approved reference.
//!
//! Three outcomes, deliberately distinct. Matching passes. Differing fails. A
//! reference whose prototype or environment no longer matches is *stale*, which
//! means the prototype or the environment moved, not that the product regressed
//! — reporting that as a regression would teach people to re-baseline reflexively.

use vulcan_domain::rendering::{Difference, Image, Viewport};
use vulcan_domain::verdict::{GateError, GateVerdict};

use crate::ports::image_compare::{CompareError, ImageComparePort};
use crate::ports::render_capture::{RenderCapturePort, RenderError};

pub struct ReferenceDigests {
    pub prototype: String,
    pub environment: String,
}

pub struct CompareInput {
    pub viewport: Viewport,
    pub reference: Image,
    pub recorded: ReferenceDigests,
    pub current: ReferenceDigests,
}

pub enum Outcome {
    Matches,
    Differs { difference: Difference, actual: Image },
    /// The reference describes a prototype or environment that no longer exists.
    Stale { reason: String },
}

pub struct CompareFidelity<C: RenderCapturePort, I: ImageComparePort> {
    capture: C,
    compare: I,
}

impl<C: RenderCapturePort, I: ImageComparePort> CompareFidelity<C, I> {
    pub fn new(capture: C, compare: I) -> Self {
        Self { capture, compare }
    }

    pub fn execute(&self, input: CompareInput) -> Result<Outcome, GateError> {
        if input.recorded.prototype != input.current.prototype {
            return Ok(Outcome::Stale {
                reason: format!(
                    "the prototype changed since this reference was captured ({} -> {}); re-approve rather than re-baseline",
                    short(&input.recorded.prototype),
                    short(&input.current.prototype)
                ),
            });
        }
        if input.recorded.environment != input.current.environment {
            return Ok(Outcome::Stale {
                reason: format!(
                    "the comparison environment changed since this reference was captured ({} -> {})",
                    short(&input.recorded.environment),
                    short(&input.current.environment)
                ),
            });
        }

        let actual = self.capture.capture(input.viewport).map_err(|error| {
            GateError::CouldNotJudge(match error {
                RenderError::NotPinnedEnvironment => {
                    "not inside the pinned environment; an exact comparison taken elsewhere is different evidence, not weaker evidence".into()
                }
                RenderError::TypefaceMissing(font) => format!("typeface missing: {font}"),
                RenderError::Failed(detail) => format!("capture failed: {detail}"),
            })
        })?;

        let difference = self.compare.compare(&actual, &input.reference).map_err(|error| {
            GateError::CouldNotJudge(match error {
                CompareError::DimensionMismatch { actual, reference } => format!(
                    "captured {}x{} against a {}x{} reference",
                    actual.0, actual.1, reference.0, reference.1
                ),
            })
        })?;

        Ok(match difference {
            Difference::None => Outcome::Matches,
            difference => Outcome::Differs { difference, actual },
        })
    }
}

pub fn verdict_of(outcome: &Outcome) -> GateVerdict {
    match outcome {
        Outcome::Matches => GateVerdict::Passed,
        Outcome::Stale { reason } => GateVerdict::Failed {
            findings: vec![format!("stale reference: {reason}")],
        },
        Outcome::Differs { difference, .. } => match difference {
            Difference::Pixels { count, first_at } => GateVerdict::Failed {
                findings: vec![format!(
                    "{count} pixels differ, first at {},{}",
                    first_at.0, first_at.1
                )],
            },
            Difference::None => GateVerdict::Passed,
        },
    }
}

fn short(digest: &str) -> String {
    digest.chars().take(12).collect()
}
