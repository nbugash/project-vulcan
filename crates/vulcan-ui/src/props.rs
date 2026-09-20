//! The six properties the prototype exposes, and the dimensions density resolves.
//!
//! Values come from the extracted token set, so a change to the prototype's
//! density table surfaces as a token diff rather than as a code edit.

use crate::generated_tokens as token;
use crate::tokens::px_of;
use vulcan_domain::rendering::Density;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfReadout {
    Status,
    Hud,
    Off,
}

/// Which overlay the shell composes. The prototype draws these states, so the
/// shell must be able to render each for capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
    Completion,
    Palette,
    RunConfig,
}

/// The rail's panels, named as the prototype names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailTab {
    Project,
    Structure,
    Commit,
    Find,
    History,
    Packs,
}

#[derive(Debug, Clone, Copy)]
pub struct Props {
    pub density: Density,
    pub tool_side: ToolSide,
    pub perf_readout: PerfReadout,
    pub show_inlay: bool,
    pub remote_banner: bool,
    pub overlay: Overlay,
    pub rail_tab: RailTab,
    pub side_collapsed: bool,
    pub dock_collapsed: bool,
    pub palette_mode: crate::palette::Mode,
    pub palette_query: &'static str,
    pub completion_style: CompletionStyle,
    /// `comp: true` in the prototype's initial state: the popup is part of the
    /// depicted composition, not a state you have to reach.
    pub completion_open: bool,
    /// Which editor tab is in front.
    pub active_tab: usize,
    /// Which dock panel is in front: terminal, debug, problems, resources.
    pub dock_panel: usize,
    /// Set by dragging an edge. `None` means the density profile decides, which
    /// is the state until someone drags something.
    pub tool_window_width: Option<f32>,
    pub dock_height: Option<f32>,
}

impl Default for Props {
    fn default() -> Self {
        Self {
            density: Density::Default,
            tool_side: ToolSide::Left,
            perf_readout: PerfReadout::Hud,
            show_inlay: true,
            remote_banner: true,
            overlay: Overlay::None,
            rail_tab: RailTab::Project,
            side_collapsed: false,
            dock_collapsed: false,
            palette_mode: crate::palette::Mode::Files,
            palette_query: "",
            completion_style: CompletionStyle::Detail,
            completion_open: true,
            active_tab: 0,
            dock_panel: 0,
            tool_window_width: None,
            dock_height: None,
        }
    }
}

/// `completionStyle` in the prototype, which widens the popup from 430px to
/// 620px so a signature and its documentation fit beside the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompletionStyle {
    Compact,
    /// What the prototype depicts: the list with its documentation panel.
    #[default]
    Detail,
}

impl CompletionStyle {
    pub fn width(self) -> f32 {
        match self {
            CompletionStyle::Compact => 430.0,
            CompletionStyle::Detail => 620.0,
        }
    }
}

/// Every dimension that changes with density, resolved from the token set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DensityProfile {
    pub code_line_height: f32,
    pub row_height: f32,
    pub ui_font_size: f32,
    pub code_font_size: f32,
    pub tool_window_width: f32,
    pub dock_height: f32,
}

impl DensityProfile {
    pub fn resolve(density: Density) -> Self {
        match density {
            Density::Compact => Self {
                code_line_height: px_of(token::VK_LINE_COMPACT),
                row_height: px_of(token::VK_ROW_COMPACT),
                ui_font_size: px_of(token::VK_FS_COMPACT),
                code_font_size: px_of(token::VK_CODE_COMPACT),
                tool_window_width: px_of(token::VK_TOOL_COMPACT),
                dock_height: px_of(token::VK_DOCK_COMPACT),
            },
            Density::Default => Self {
                code_line_height: px_of(token::VK_LINE_DEFAULT),
                row_height: px_of(token::VK_ROW_DEFAULT),
                ui_font_size: px_of(token::VK_FS_DEFAULT),
                code_font_size: px_of(token::VK_CODE_DEFAULT),
                tool_window_width: px_of(token::VK_TOOL_DEFAULT),
                dock_height: px_of(token::VK_DOCK_DEFAULT),
            },
            Density::Roomy => Self {
                code_line_height: px_of(token::VK_LINE_ROOMY),
                row_height: px_of(token::VK_ROW_ROOMY),
                ui_font_size: px_of(token::VK_FS_ROOMY),
                code_font_size: px_of(token::VK_CODE_ROOMY),
                tool_window_width: px_of(token::VK_TOOL_ROOMY),
                dock_height: px_of(token::VK_DOCK_ROOMY),
            },
        }
    }

    /// The dock is capped so the editor always keeps room, as the prototype states.
    pub fn dock_height_for(&self, viewport_height: f32) -> f32 {
        self.dock_height.min(viewport_height * 0.34)
    }
}
