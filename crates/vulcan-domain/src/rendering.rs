//! Types the fidelity comparison works in.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub viewport: Viewport,
    /// Row-major RGBA8.
    pub pixels: Vec<u8>,
}

impl Image {
    /// How many distinct colours the image contains, counting no further than
    /// `limit`.
    ///
    /// This exists to tell a render from the absence of one. A capture of a
    /// compositor whose client never drew is a single flat colour, and the
    /// comparison reports it as every pixel differing — a true statement that
    /// names the wrong cause. Capture refuses such an image instead, because
    /// the reference it would otherwise sign off is a photograph of nothing.
    ///
    /// Stops early: the answer is only ever compared against a small floor, and
    /// a real interface reaches it within the first few rows.
    pub fn distinct_colours(&self, limit: usize) -> usize {
        let mut seen: Vec<[u8; 4]> = Vec::with_capacity(limit);
        for pixel in self.pixels.chunks_exact(4) {
            let colour = [pixel[0], pixel[1], pixel[2], pixel[3]];
            if !seen.contains(&colour) {
                seen.push(colour);
                if seen.len() >= limit {
                    return seen.len();
                }
            }
        }
        seen.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Difference {
    None,
    /// Exact comparison: any differing pixel is a difference. The count and the
    /// first location are carried so the report can say what moved.
    Pixels { count: usize, first_at: (u32, u32) },
}

impl Difference {
    pub fn is_none(&self) -> bool {
        matches!(self, Difference::None)
    }
}

/// Density profiles, resolved from the extracted token set rather than written
/// in code, so a prototype change surfaces as a token diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Density {
    Compact,
    Default,
    Roomy,
}

impl Density {
    pub const ALL: [Density; 3] = [Density::Compact, Density::Default, Density::Roomy];

    pub fn token_suffix(self) -> &'static str {
        match self {
            Density::Compact => "compact",
            Density::Default => "default",
            Density::Roomy => "roomy",
        }
    }
}
