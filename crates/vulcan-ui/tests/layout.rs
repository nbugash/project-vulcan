//! T078: chrome dimensions come from the prototype, not from code.

use vulcan_ui::tokens::{px_of, rgb_of, Chrome, Palette};

#[test]
fn fixed_chrome_heights_match_the_prototype() {
    assert_eq!(Chrome::toolbar(), 46.0);
    assert_eq!(Chrome::remote_banner(), 26.0);
    assert_eq!(Chrome::tab_strip(), 34.0);
    assert_eq!(Chrome::breadcrumbs(), 24.0);
    assert_eq!(Chrome::dock_header(), 30.0);
    assert_eq!(Chrome::dock_tab_strip(), 28.0);
    assert_eq!(Chrome::status_bar(), 26.0);
    assert_eq!(Chrome::rail_width(), 44.0);
}

#[test]
fn the_palette_resolves_the_design_system_colours() {
    assert_eq!(Palette::bg(), 0x161826);
    assert_eq!(Palette::surface(), 0x232532);
    assert_eq!(Palette::panel(), 0x292b31);
    assert_eq!(Palette::text(), 0xe9e9ed);
    assert_eq!(Palette::accent(), 0x9184d9);
}

#[test]
fn a_value_that_is_not_a_length_is_rejected() {
    assert!(std::panic::catch_unwind(|| px_of("Inter")).is_err());
    assert!(std::panic::catch_unwind(|| rgb_of("not-a-colour")).is_err());
}
