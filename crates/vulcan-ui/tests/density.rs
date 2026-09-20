//! T079: density resolution against the values the prototype's manifest declares.

use vulcan_domain::rendering::Density;
use vulcan_ui::props::DensityProfile;

#[test]
fn each_profile_resolves_the_manifest_values() {
    let compact = DensityProfile::resolve(Density::Compact);
    assert_eq!(compact.code_line_height, 19.0);
    assert_eq!(compact.row_height, 22.0);
    assert_eq!(compact.tool_window_width, 250.0);
    assert_eq!(compact.dock_height, 206.0);

    let default = DensityProfile::resolve(Density::Default);
    assert_eq!(default.code_line_height, 21.0);
    assert_eq!(default.row_height, 25.0);
    assert_eq!(default.tool_window_width, 276.0);
    assert_eq!(default.dock_height, 236.0);

    let roomy = DensityProfile::resolve(Density::Roomy);
    assert_eq!(roomy.code_line_height, 25.0);
    assert_eq!(roomy.row_height, 30.0);
    assert_eq!(roomy.tool_window_width, 310.0);
    assert_eq!(roomy.dock_height, 268.0);
}

#[test]
fn every_dimension_grows_with_density() {
    let profiles: Vec<_> = Density::ALL.iter().map(|d| DensityProfile::resolve(*d)).collect();
    for pair in profiles.windows(2) {
        assert!(pair[1].code_line_height > pair[0].code_line_height);
        assert!(pair[1].row_height > pair[0].row_height);
        assert!(pair[1].tool_window_width > pair[0].tool_window_width);
        assert!(pair[1].dock_height > pair[0].dock_height);
    }
}

#[test]
fn the_dock_is_capped_so_the_editor_keeps_room() {
    let profile = DensityProfile::resolve(Density::Roomy);
    // On a short viewport the cap binds, as the prototype states.
    assert_eq!(profile.dock_height_for(600.0), 204.0);
    // On a tall one the declared height wins.
    assert_eq!(profile.dock_height_for(1200.0), 268.0);
}
