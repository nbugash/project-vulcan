//! T112: the shell responds to input.
//!
//! These test the transitions a click performs, not the click plumbing. GPUI
//! offers no way to inject a synthetic click without a window and a compositor,
//! so the listeners are kept to one line each — `shell.select_rail(tab)` — and
//! the behaviour those lines invoke is tested here.

use vulcan_ui::palette::Mode;
use vulcan_ui::props::{Overlay, Props, RailTab, ToolSide};
use vulcan_ui::shell::Shell;

fn shell() -> Shell {
    Shell::new(Props::default())
}

#[test]
fn selecting_a_rail_destination_shows_it() {
    let mut shell = shell();
    shell.select_rail(RailTab::Commit);
    assert_eq!(shell.props().rail_tab, RailTab::Commit);
    assert!(!shell.props().side_collapsed);
}

#[test]
fn selecting_the_destination_already_showing_collapses_the_tool_window() {
    // The prototype's rail toggles: clicking the active destination closes the
    // panel rather than doing nothing.
    let mut shell = shell();
    let showing = shell.props().rail_tab;
    shell.select_rail(showing);
    assert!(shell.props().side_collapsed);
}

#[test]
fn selecting_a_different_destination_reopens_a_collapsed_tool_window() {
    let mut shell = shell();
    shell.collapse_tool_window(true);
    shell.select_rail(RailTab::Find);
    assert_eq!(shell.props().rail_tab, RailTab::Find);
    assert!(!shell.props().side_collapsed, "a destination with no panel visible is a dead end");
}

#[test]
fn the_dock_tab_strip_toggles_the_same_way() {
    let mut shell = shell();
    shell.select_dock_panel(2);
    assert_eq!(shell.props().dock_panel, 2);
    assert!(!shell.props().dock_collapsed);

    shell.select_dock_panel(2);
    assert!(shell.props().dock_collapsed, "the open panel's own tab closes it");

    shell.select_dock_panel(2);
    assert!(!shell.props().dock_collapsed, "and opens it again");
}

#[test]
fn the_palette_opens_on_the_mode_asked_for_and_closes() {
    let mut shell = shell();
    assert_eq!(shell.props().overlay, Overlay::None);

    shell.open_palette(Mode::Commands);
    assert_eq!(shell.props().overlay, Overlay::Palette);
    assert_eq!(shell.props().palette_mode, Mode::Commands);

    shell.close_overlay();
    assert_eq!(shell.props().overlay, Overlay::None);
}

#[test]
fn switching_palette_mode_keeps_it_open() {
    let mut shell = shell();
    shell.open_palette(Mode::Files);
    shell.open_palette(Mode::Structural);
    assert_eq!(shell.props().overlay, Overlay::Palette);
    assert_eq!(shell.props().palette_mode, Mode::Structural);
}

#[test]
fn an_editor_tab_can_be_brought_to_the_front() {
    let mut shell = shell();
    assert_eq!(shell.props().active_tab, 0);
    shell.select_tab(2);
    assert_eq!(shell.props().active_tab, 2);
}

#[test]
fn the_tool_window_moves_to_the_other_side_and_back() {
    let mut shell = shell();
    let first = shell.props().tool_side;
    shell.toggle_tool_side();
    assert_ne!(shell.props().tool_side, first);
    shell.toggle_tool_side();
    assert_eq!(shell.props().tool_side, first);
}

#[test]
fn every_rail_destination_is_reachable() {
    // A rail tab that cannot be selected is a control that lies about itself.
    let mut shell = shell();
    for tab in [
        RailTab::Project,
        RailTab::Structure,
        RailTab::Commit,
        RailTab::Find,
        RailTab::History,
        RailTab::Packs,
    ] {
        shell.collapse_tool_window(false);
        shell.select_rail(tab);
        assert_eq!(shell.props().rail_tab, tab, "{tab:?} is not reachable");
    }
}

#[test]
fn tool_side_is_the_only_thing_toggling_it_changes() {
    let mut shell = shell();
    let before = (shell.props().rail_tab, shell.props().overlay, shell.props().active_tab);
    shell.toggle_tool_side();
    let after = (shell.props().rail_tab, shell.props().overlay, shell.props().active_tab);
    assert_eq!(before, after, "moving the tool window disturbed unrelated state");
}

/// T114: density, tool side and the performance readout change at runtime.
mod runtime_props {
    use super::*;
    use vulcan_domain::rendering::Density;
    use vulcan_ui::props::PerfReadout;

    #[test]
    fn changing_density_re_resolves_every_dependent_dimension() {
        // Recording the choice without re-resolving the profile would leave the
        // shell drawing the previous density's dimensions.
        let mut shell = shell();
        let before = *shell.profile();

        shell.set_density(Density::Compact);
        let compact = *shell.profile();
        assert_ne!(compact.code_line_height, before.code_line_height);
        assert_ne!(compact.row_height, before.row_height);

        shell.set_density(Density::Roomy);
        let roomy = *shell.profile();
        assert!(roomy.code_line_height > compact.code_line_height);
    }

    #[test]
    fn density_cycles_through_all_three_and_returns() {
        let mut shell = shell();
        let start = shell.props().density;
        let seen: Vec<Density> = (0..3)
            .map(|_| {
                shell.cycle_density();
                shell.props().density
            })
            .collect();
        assert_eq!(seen.len(), 3);
        assert_eq!(shell.props().density, start, "cycling three times returns to the start");
        let mut distinct = seen.clone();
        distinct.dedup();
        assert_eq!(distinct.len(), 3, "every density is reachable: {seen:?}");
    }

    #[test]
    fn the_latency_readout_shows_and_hides_the_frame_panel() {
        let mut shell = shell();
        assert_eq!(shell.props().perf_readout, PerfReadout::Hud);
        shell.toggle_hud();
        assert_eq!(shell.props().perf_readout, PerfReadout::Status);
        shell.toggle_hud();
        assert_eq!(shell.props().perf_readout, PerfReadout::Hud);
    }
}
