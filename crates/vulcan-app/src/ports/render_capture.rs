use vulcan_domain::rendering::{Image, Viewport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    /// Exact comparison is only meaningful where rasterisation is fixed.
    NotPinnedEnvironment,
    TypefaceMissing(String),
    Failed(String),
}

pub trait RenderCapturePort {
    fn capture(&self, viewport: Viewport) -> Result<Image, RenderError>;
}
