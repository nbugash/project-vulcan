//! T050: the fidelity use cases against fakes.

use vulcan_app::ports::image_compare::{CompareError, ImageComparePort};
use vulcan_app::ports::render_capture::{RenderCapturePort, RenderError};
use vulcan_app::use_cases::compare_fidelity::{
    CompareFidelity, CompareInput, Outcome, ReferenceDigests,
};
use vulcan_app::use_cases::lint_off_token::{Literal, LintOffToken};
use vulcan_domain::design_value::{DesignValue, TokenSet, ValueKind, ValueSource};
use vulcan_domain::rendering::{Difference, Image, Viewport};
use vulcan_domain::verdict::GateError;

fn viewport() -> Viewport {
    Viewport { width: 4, height: 1 }
}

fn image(fill: u8) -> Image {
    Image { viewport: viewport(), pixels: vec![fill; 16] }
}

struct FakeCapture(Result<Image, RenderError>);
impl RenderCapturePort for FakeCapture {
    fn capture(&self, _: Viewport) -> Result<Image, RenderError> {
        self.0.clone().map_err(|e| e)
    }
}

struct FakeCompare(Difference);
impl ImageComparePort for FakeCompare {
    fn compare(&self, _: &Image, _: &Image) -> Result<Difference, CompareError> {
        Ok(self.0.clone())
    }
}

fn digests(prototype: &str, environment: &str) -> ReferenceDigests {
    ReferenceDigests { prototype: prototype.into(), environment: environment.into() }
}

fn input(recorded: (&str, &str), current: (&str, &str)) -> CompareInput {
    CompareInput {
        viewport: viewport(),
        reference: image(0),
        recorded: digests(recorded.0, recorded.1),
        current: digests(current.0, current.1),
    }
}

#[test]
fn an_identical_rendering_matches() {
    let use_case = CompareFidelity::new(FakeCapture(Ok(image(0))), FakeCompare(Difference::None));
    assert!(matches!(
        use_case.execute(input(("p1", "e1"), ("p1", "e1"))).unwrap(),
        Outcome::Matches
    ));
}

#[test]
fn any_differing_pixel_fails() {
    let difference = Difference::Pixels { count: 1, first_at: (2, 0) };
    let use_case = CompareFidelity::new(FakeCapture(Ok(image(9))), FakeCompare(difference));
    assert!(matches!(
        use_case.execute(input(("p1", "e1"), ("p1", "e1"))).unwrap(),
        Outcome::Differs { .. }
    ));
}

#[test]
fn a_changed_prototype_is_stale_not_a_regression() {
    let use_case = CompareFidelity::new(FakeCapture(Ok(image(0))), FakeCompare(Difference::None));
    match use_case.execute(input(("old", "e1"), ("new", "e1"))).unwrap() {
        Outcome::Stale { reason } => assert!(reason.contains("prototype changed"), "{reason}"),
        _ => panic!("expected Stale"),
    }
}

#[test]
fn a_changed_environment_is_stale_too() {
    let use_case = CompareFidelity::new(FakeCapture(Ok(image(0))), FakeCompare(Difference::None));
    match use_case.execute(input(("p1", "old"), ("p1", "new"))).unwrap() {
        Outcome::Stale { reason } => assert!(reason.contains("environment changed"), "{reason}"),
        _ => panic!("expected Stale"),
    }
}

#[test]
fn comparing_outside_the_pinned_environment_cannot_be_judged() {
    let use_case = CompareFidelity::new(
        FakeCapture(Err(RenderError::NotPinnedEnvironment)),
        FakeCompare(Difference::None),
    );
    match use_case.execute(input(("p1", "e1"), ("p1", "e1"))) {
        Err(GateError::CouldNotJudge(reason)) => assert!(reason.contains("pinned"), "{reason}"),
        _ => panic!("expected CouldNotJudge"),
    }
}

#[test]
fn a_missing_typeface_cannot_be_judged() {
    let use_case = CompareFidelity::new(
        FakeCapture(Err(RenderError::TypefaceMissing("Inter".into()))),
        FakeCompare(Difference::None),
    );
    assert!(use_case.execute(input(("p1", "e1"), ("p1", "e1"))).is_err());
}

fn token_set() -> TokenSet {
    TokenSet::build(vec![DesignValue {
        name: "color-accent".into(),
        value: "#9184d9".into(),
        kind: ValueKind::Colour,
        source: ValueSource::Stylesheet,
        source_location: "styles.css:42".into(),
    }])
    .unwrap()
}

#[test]
fn an_extracted_value_passes_the_lint() {
    let found = vec![Literal { value: "#9184d9".into(), file: "shell.rs".into(), line: 10 }];
    assert!(!LintOffToken::execute(&token_set(), &[], &found).is_blocking());
}

#[test]
fn a_literal_outside_the_set_fails_and_names_its_location() {
    let found = vec![Literal { value: "#ff0000".into(), file: "shell.rs".into(), line: 12 }];
    let verdict = LintOffToken::execute(&token_set(), &[], &found);
    assert!(verdict.is_blocking());
    assert!(verdict.findings()[0].contains("#ff0000"));
    assert!(verdict.findings()[0].contains("shell.rs:12"));
}

/// A flat image is what a capture of a compositor whose client never drew looks
/// like. The pinned image once pointed `VK_ICD_FILENAMES` at a filename its own
/// distribution does not use; the loader enumerated no driver, the shell drew
/// nothing, and `capture-reference` signed the blank result off as the
/// reference. Counting colours is how capture tells drawing from not drawing.
#[test]
fn a_flat_image_holds_one_colour() {
    assert_eq!(image(0).distinct_colours(16), 1);
}

#[test]
fn counting_colours_stops_at_the_limit_it_is_given() {
    let pixels: Vec<u8> = (0u8..64).collect();
    let image = Image { viewport: Viewport { width: 16, height: 1 }, pixels };
    assert_eq!(image.distinct_colours(16), 16);
    assert_eq!(image.distinct_colours(4), 4);
}
