//! T051: the exact comparator, against real images.

use vulcan_adapters::rendering::exact_compare::ExactImageComparator;
use vulcan_app::ports::image_compare::{CompareError, ImageComparePort};
use vulcan_domain::rendering::{Difference, Image, Viewport};

fn image(width: u32, height: u32, fill: u8) -> Image {
    Image {
        viewport: Viewport { width, height },
        pixels: vec![fill; (width * height * 4) as usize],
    }
}

#[test]
fn identical_images_have_no_difference() {
    assert_eq!(
        ExactImageComparator.compare(&image(4, 4, 7), &image(4, 4, 7)).unwrap(),
        Difference::None
    );
}

#[test]
fn a_single_differing_pixel_is_found_and_located() {
    let mut actual = image(4, 2, 0);
    actual.pixels[(1 * 4 + 2) * 4] = 255;
    match ExactImageComparator.compare(&actual, &image(4, 2, 0)).unwrap() {
        Difference::Pixels { count, first_at } => {
            assert_eq!(count, 1);
            assert_eq!(first_at, (2, 1));
        }
        Difference::None => panic!("a differing pixel must be detected; that is the whole gate"),
    }
}

#[test]
fn mismatched_dimensions_cannot_be_compared() {
    match ExactImageComparator.compare(&image(4, 4, 0), &image(5, 4, 0)) {
        Err(CompareError::DimensionMismatch { actual, reference }) => {
            assert_eq!(actual, (4, 4));
            assert_eq!(reference, (5, 4));
        }
        _ => panic!("expected DimensionMismatch"),
    }
}
