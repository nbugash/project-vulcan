//! What changed, as an image rather than a number.
//!
//! A count of differing pixels says a comparison failed; it does not say where.
//! The report dims what matched and marks what did not, so a reviewer sees the
//! change at a glance.

use std::path::Path;

use vulcan_domain::rendering::Image;

use super::reference_store::{write_png, ReferenceError};

/// Colour used to mark differing pixels: the prototype's error colour.
const MARK: [u8; 4] = [0xd4, 0x73, 0x6a, 0xff];

pub fn write_difference(path: &Path, actual: &Image, reference: &Image) -> Result<usize, ReferenceError> {
    let mut pixels = Vec::with_capacity(actual.pixels.len());
    let mut marked = 0usize;

    for (a, b) in actual.pixels.chunks_exact(4).zip(reference.pixels.chunks_exact(4)) {
        if a == b {
            // Dim what matched so the marks carry the eye.
            pixels.extend_from_slice(&[a[0] / 3, a[1] / 3, a[2] / 3, 0xff]);
        } else {
            pixels.extend_from_slice(&MARK);
            marked += 1;
        }
    }

    write_png(path, &Image { viewport: actual.viewport, pixels })?;
    Ok(marked)
}
