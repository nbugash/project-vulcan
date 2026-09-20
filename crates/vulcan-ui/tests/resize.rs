//! T113: the tool window and dock resize by dragging, within declared bounds.
//!
//! "Within the widths the manifest declares" is the constraint that matters. The
//! prototype gives three widths per surface, one per density, and those are the
//! only widths it sanctions. A drag may land anywhere between the narrowest and
//! the widest; it may not invent a width outside them, because a value the
//! prototype never states is a design value this product made up.

use vulcan_ui::generated_tokens as token;
use vulcan_ui::props::Props;
use vulcan_ui::shell::Shell;
use vulcan_ui::tokens::px_of;

fn shell() -> Shell {
    Shell::new(Props::default())
}

fn tool_bounds() -> (f32, f32) {
    (px_of(token::VK_TOOL_COMPACT), px_of(token::VK_TOOL_ROOMY))
}

fn dock_bounds() -> (f32, f32) {
    (px_of(token::VK_DOCK_COMPACT), px_of(token::VK_DOCK_ROOMY))
}

#[test]
fn dragging_the_tool_window_edge_widens_it() {
    let mut shell = shell();
    let before = shell.tool_window_width();
    shell.resize_tool_window(20.0);
    assert!(
        shell.tool_window_width() > before,
        "dragging outward by 20px left the width at {before}"
    );
}

#[test]
fn dragging_inward_narrows_it() {
    let mut shell = shell();
    let before = shell.tool_window_width();
    shell.resize_tool_window(-15.0);
    assert!(shell.tool_window_width() < before);
}

#[test]
fn the_tool_window_cannot_be_dragged_wider_than_the_manifest_allows() {
    let (_, widest) = tool_bounds();
    let mut shell = shell();
    shell.resize_tool_window(10_000.0);
    assert_eq!(
        shell.tool_window_width(),
        widest,
        "a drag past the widest declared width must stop at it"
    );
}

#[test]
fn the_tool_window_cannot_be_dragged_narrower_than_the_manifest_allows() {
    let (narrowest, _) = tool_bounds();
    let mut shell = shell();
    shell.resize_tool_window(-10_000.0);
    assert_eq!(shell.tool_window_width(), narrowest);
}

#[test]
fn the_dock_resizes_within_its_own_declared_heights() {
    let (shortest, tallest) = dock_bounds();
    let mut shell = shell();

    shell.resize_dock(10_000.0);
    assert_eq!(shell.dock_height(), tallest);

    shell.resize_dock(-10_000.0);
    assert_eq!(shell.dock_height(), shortest);
}

#[test]
fn a_drag_is_cumulative_rather_than_absolute() {
    // A drag arrives as a series of small deltas, so each must move the edge
    // from where the last one left it.
    let mut shell = shell();
    let before = shell.tool_window_width();
    for _ in 0..5 {
        shell.resize_tool_window(4.0);
    }
    assert!((shell.tool_window_width() - (before + 20.0)).abs() < 0.01);
}

#[test]
fn resizing_changes_nothing_but_the_edge_that_was_dragged() {
    let mut shell = shell();
    let dock = shell.dock_height();
    let line = shell.profile().code_line_height;
    let row = shell.profile().row_height;

    shell.resize_tool_window(18.0);

    assert_eq!(shell.dock_height(), dock, "resizing the tool window moved the dock");
    assert_eq!(shell.profile().code_line_height, line);
    assert_eq!(shell.profile().row_height, row);
}

#[test]
fn a_resized_width_survives_a_density_change_only_if_it_is_still_allowed() {
    // Density and a drag both set the same dimension. The drag is the more
    // recent instruction, so it wins, but it is still bound by the manifest.
    let mut shell = shell();
    shell.resize_tool_window(10_000.0);
    let widest = shell.tool_window_width();

    shell.set_density(vulcan_domain::rendering::Density::Compact);
    assert!(
        shell.tool_window_width() <= widest,
        "a density change must not widen past what the drag left"
    );
}
