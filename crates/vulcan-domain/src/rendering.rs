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
