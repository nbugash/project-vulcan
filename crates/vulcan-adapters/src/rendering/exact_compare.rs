//! Exact image comparison: any differing pixel is a difference.
//!
//! Tolerance would make the check survive a moved environment, but it would also
//! tolerate a one-pixel spacing shift, which is the thing the gate exists to
//! catch. Stability comes from pinning the environment instead.

use vulcan_app::ports::image_compare::{CompareError, ImageComparePort};
use vulcan_domain::rendering::{Difference, Image};

pub struct ExactImageComparator;

impl ImageComparePort for ExactImageComparator {
    fn compare(&self, actual: &Image, reference: &Image) -> Result<Difference, CompareError> {
        if actual.viewport != reference.viewport {
            return Err(CompareError::DimensionMismatch {
                actual: (actual.viewport.width, actual.viewport.height),
                reference: (reference.viewport.width, reference.viewport.height),
            });
        }

        let mut count = 0usize;
        let mut first_at = None;
        let width = actual.viewport.width as usize;

        for (index, (a, b)) in actual
            .pixels
            .chunks_exact(4)
            .zip(reference.pixels.chunks_exact(4))
            .enumerate()
        {
            if a != b {
                count += 1;
                if first_at.is_none() {
                    first_at = Some(((index % width) as u32, (index / width) as u32));
                }
            }
        }

        Ok(match first_at {
            None => Difference::None,
            Some(at) => Difference::Pixels { count, first_at: at },
        })
    }
}
