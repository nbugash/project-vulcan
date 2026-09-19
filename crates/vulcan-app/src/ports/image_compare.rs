use vulcan_domain::rendering::{Difference, Image};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareError {
    DimensionMismatch { actual: (u32, u32), reference: (u32, u32) },
}

pub trait ImageComparePort {
    fn compare(&self, actual: &Image, reference: &Image) -> Result<Difference, CompareError>;
}
