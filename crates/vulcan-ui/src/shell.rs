//! The application shell: window, chrome and regions.
//!
//! Every dimension and colour here comes from `tokens`, which reads values the
//! fidelity gate extracted from the signed-off prototype. Nothing is invented,
//! which is what lets the off-token lint reject anything that is.
//!
//! Controls respond visually and dispatch to the command sink, which does
//! nothing in this feature. A control that partly acts cannot be told apart from
//! a defect by the feature that later owns the behaviour (FR-031).

use gpui::prelude::*;
use gpui::{anchored, deferred, div, px, relative, rgb, rgba, Context, SharedString, Window};


use crate::fixture;
use crate::fonts;
use crate::icons::Icon;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use vulcan_app::ports::frame_recorder::{FrameRecorderPort, NotRecording};
use vulcan_domain::budget::Metric;

use crate::props::{CompletionStyle, DensityProfile, Overlay, PerfReadout, Props, RailTab, ToolSide};
use vulcan_domain::rendering::Density;
use crate::palette::{count_label, matching, Mode, Tint};
use crate::tokens::{rgb_of, Chrome, Palette};

/// A file's version-control state, shown as the letter beside its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vcs {
    Added,
    Modified,
}

/// Where every control activation goes in this feature: nowhere.
/// F002 replaces the implementation, not the call site.
#[derive(Clone, Default)]
pub struct NoOpCommandSink;

impl NoOpCommandSink {
    pub fn dispatch(&self, _command: &'static str) {}
}

/// The thing a resize drag carries, which is nothing: the gesture moves an
/// edge rather than transporting anything.
struct DragHandle;

impl Render for DragHandle {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub struct Shell {
    props: Props,
    pub(crate) profile: DensityProfile,
    commands: NoOpCommandSink,
    /// Where frame and input timings go. `NotRecording` on an ordinary run, so
    /// the measured path and the run path are the same path.
    recorder: Arc<dyn FrameRecorderPort>,
    /// When the most recent input was handled, so the frame that follows it can
    /// be attributed to it.
    pending_input: Mutex<Option<Instant>>,
}

impl Shell {
    pub fn new(props: Props) -> Self {
        Self::measured(props, Arc::new(NotRecording))
    }

    /// The same shell, reporting its timings. The composition root decides.
    pub fn measured(props: Props, recorder: Arc<dyn FrameRecorderPort>) -> Self {
        Self {
            profile: DensityProfile::resolve(props.density),
            props,
            commands: NoOpCommandSink,
            recorder,
            pending_input: Mutex::new(None),
        }
    }

    /// Called as an input is handled, so the next frame can be measured from it.
    fn note_input(&self) {
        if let Ok(mut slot) = self.pending_input.lock() {
            // Keep the earliest unpainted input: the budget is the wait the user
            // actually experienced, not the wait since the last of several.
            slot.get_or_insert_with(Instant::now);
        }
    }

    pub fn props(&self) -> &Props {
        &self.props
    }

    /// The tool window's width: what a drag left, or what the density says.
    pub fn tool_window_width(&self) -> f32 {
        self.props.tool_window_width.unwrap_or(self.profile.tool_window_width)
    }

    pub fn dock_height(&self) -> f32 {
        self.props.dock_height.unwrap_or(self.profile.dock_height)
    }

    /// The dock's height, still capped so the editor keeps its share of the
    /// window however far the edge was dragged.
    pub fn dock_height_for(&self, viewport_height: f32) -> f32 {
        self.dock_height().min(viewport_height * 0.34)
    }

    /// Puts the tool window's edge at an absolute position, which is where the
    /// pointer is rather than how far it has travelled.
    ///
    /// Following the pointer directly means a drag cannot drift: accumulating
    /// deltas loses a pixel wherever one is dropped, and the edge ends up
    /// somewhere the pointer is not.
    pub fn place_tool_window_edge(&mut self, at: f32) {
        let width = at - Chrome::rail_width();
        self.resize_tool_window(width - self.tool_window_width());
    }

    /// The dock grows upward, so its height is the distance from the pointer to
    /// the bottom of the window.
    pub fn place_dock_edge(&mut self, at: f32, viewport_height: f32) {
        let height = viewport_height - Chrome::status_bar() - at;
        self.resize_dock(height - self.dock_height());
    }

    /// A draggable edge. Four pixels, which is the divider width the prototype
    /// uses, and a cursor that says which way it moves before anyone commits to
    /// finding out.
    fn resize_handle(
        id: &'static str,
        vertical: bool,
        cx: &mut Context<Self>,
        place: impl Fn(&mut Self, f32) + 'static,
    ) -> impl IntoElement {
        div()
            .id(id)
            .when(vertical, |handle| handle.w(px(4.0)).h_full().cursor_col_resize())
            .when(!vertical, |handle| handle.h(px(4.0)).w_full().cursor_row_resize())
            .bg(rgb(Palette::divider()))
            // The payload is unit: what is being dragged is the edge itself, and
            // nothing is being carried anywhere.
            .on_drag((), |_, _, _, cx| cx.new(|_| DragHandle))
            .on_drag_move(cx.listener(
                move |shell, event: &gpui::DragMoveEvent<()>, _window, cx| {
                    let at: f32 = if vertical {
                        event.event.position.x.into()
                    } else {
                        event.event.position.y.into()
                    };
                    place(shell, at);
                    cx.notify();
                },
            ))
    }

    /// Moves the tool window's edge, bounded by the widths the prototype states.
    ///
    /// The manifest gives three widths per surface, one per density, and those
    /// are the only widths it sanctions. A drag may land anywhere between the
    /// narrowest and the widest; beyond them it stops, because a width the
    /// prototype never states is a design value this product invented.
    pub fn resize_tool_window(&mut self, delta: f32) {
        let narrowest = crate::tokens::px_of(crate::generated_tokens::VK_TOOL_COMPACT);
        let widest = crate::tokens::px_of(crate::generated_tokens::VK_TOOL_ROOMY);
        self.props.tool_window_width =
            Some((self.tool_window_width() + delta).clamp(narrowest, widest));
        self.commands.dispatch("tool.resize");
        self.note_input();
    }

    pub fn resize_dock(&mut self, delta: f32) {
        let shortest = crate::tokens::px_of(crate::generated_tokens::VK_DOCK_COMPACT);
        let tallest = crate::tokens::px_of(crate::generated_tokens::VK_DOCK_ROOMY);
        self.props.dock_height = Some((self.dock_height() + delta).clamp(shortest, tallest));
        self.commands.dispatch("dock.resize");
        self.note_input();
    }

    /// Density drives every dimension in the profile, so changing it has to
    /// re-resolve the profile rather than only record the choice.
    pub fn set_density(&mut self, density: Density) {
        self.props.density = density;
        self.profile = DensityProfile::resolve(density);
        self.commands.dispatch("density.set");
        self.note_input();
    }

    pub fn cycle_density(&mut self) {
        self.set_density(match self.props.density {
            Density::Compact => Density::Default,
            Density::Default => Density::Roomy,
            Density::Roomy => Density::Compact,
        });
    }

    /// `toggleHud` in the prototype: the status bar's latency readout is the
    /// control that shows and hides the frame-timing panel.
    pub fn toggle_hud(&mut self) {
        self.props.perf_readout = match self.props.perf_readout {
            PerfReadout::Hud => PerfReadout::Status,
            PerfReadout::Status => PerfReadout::Hud,
            PerfReadout::Off => PerfReadout::Hud,
        };
        self.commands.dispatch("perf.toggle");
        self.note_input();
    }

    /// Selecting the rail destination that is already showing collapses the
    /// tool window, which is how the prototype's rail behaves.
    pub fn select_rail(&mut self, tab: RailTab) {
        if self.props.rail_tab == tab && !self.props.side_collapsed {
            self.props.side_collapsed = true;
        } else {
            self.props.rail_tab = tab;
            self.props.side_collapsed = false;
        }
        self.commands.dispatch("rail.select");
        self.note_input();
    }

    /// The dock's tab strip behaves the same way: the open panel's tab closes it.
    pub fn select_dock_panel(&mut self, index: usize) {
        if self.props.dock_panel == index && !self.props.dock_collapsed {
            self.props.dock_collapsed = true;
        } else {
            self.props.dock_panel = index;
            self.props.dock_collapsed = false;
        }
        self.commands.dispatch("dock.panel");
        self.note_input();
    }

    pub fn open_palette(&mut self, mode: Mode) {
        self.props.overlay = Overlay::Palette;
        self.props.palette_mode = mode;
        self.commands.dispatch("palette.open");
        self.note_input();
    }

    pub fn close_overlay(&mut self) {
        self.props.overlay = Overlay::None;
        self.commands.dispatch("overlay.close");
        self.note_input();
    }

    pub fn select_tab(&mut self, index: usize) {
        self.props.active_tab = index;
        self.commands.dispatch("editor.tab");
        self.note_input();
    }

    pub fn toggle_tool_side(&mut self) {
        self.props.tool_side = match self.props.tool_side {
            ToolSide::Left => ToolSide::Right,
            ToolSide::Right => ToolSide::Left,
        };
        self.commands.dispatch("tool.side");
        self.note_input();
    }

    pub fn collapse_tool_window(&mut self, collapsed: bool) {
        self.props.side_collapsed = collapsed;
        self.commands.dispatch("tool.collapse");
        self.note_input();
    }

    pub(crate) fn set_dock_collapsed(&mut self, collapsed: bool) {
        self.props.dock_collapsed = collapsed;
        self.commands.dispatch("dock.collapse");
    }

    pub(crate) fn set_dock_panel(&mut self, index: usize) {
        self.props.dock_panel = index;
        self.commands.dispatch("dock.panel");
        self.note_input();
    }

    pub(crate) fn clickable_panel(
        id: impl Into<SharedString>,
        cx: &mut Context<Self>,
        action: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> gpui::Stateful<gpui::Div> {
        Self::clickable(id, cx, action)
    }

    pub fn profile(&self) -> &DensityProfile {
        &self.profile
    }

    pub(crate) fn label(text: &str, size: f32, colour: u32) -> impl IntoElement {
        div()
            .font_family(fonts::UI_FAMILY)
            .text_size(px(size))
            .text_color(rgb(colour))
            .child(SharedString::from(text.to_string()))
    }

    /// Uppercase section labels carry the letter-spacing the prototype declares.
    pub(crate) fn section_label(text: &str, colour: u32) -> impl IntoElement {
        div()
            .font_family(fonts::UI_MEDIUM_FAMILY)
            .text_size(px(10.5))
            .text_color(rgb(colour))
            .child(SharedString::from(text.to_uppercase()))
    }

    pub(crate) fn code(text: &str, size: f32, colour: u32) -> impl IntoElement {
        div()
            .font_family(fonts::CODE_FAMILY)
            .text_size(px(size))
            .text_color(rgb(colour))
            .child(SharedString::from(text.to_string()))
    }

    /// A Phosphor glyph. The prototype draws every icon from this font, sized
    /// between 8px and 17px and coloured like text.
    pub(crate) fn icon(glyph: &'static str, size: f32, colour: u32) -> impl IntoElement {
        div()
            .font_family(fonts::ICON_FAMILY)
            .text_size(px(size))
            .text_color(rgb(colour))
            .child(SharedString::from(glyph))
    }

    /// Toolbar: 46px, on the surface colour, with the wordmark and run controls.
    fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(Chrome::toolbar()))
            .w_full()
            .flex()
            .items_center()
            .gap(px(14.0))
            .px(px(12.0))
            .bg(rgb(Palette::surface()))
            .child(Self::icon(Icon::CUBE, 16.0, Palette::accent()))
            .child(
                div()
                    .font_family(fonts::UI_MEDIUM_FAMILY)
                    .text_size(px(11.0))
                    .text_color(rgb(Palette::text()))
                    .child(SharedString::from("VULCAN")),
            )
            .child(Self::label(fixture::PROJECT, 12.0, Palette::muted()))
            .child(Self::icon(Icon::CARET_DOWN, 10.0, Palette::dim()))
            .child(
                // The dropdown is a child of the control, so its left edge
                // aligns with the control's by construction rather than by a
                // hand-tuned offset that drifts when the toolbar changes.
                div()
                    .relative()
                    .child(self.run_config_button())
                    .when(self.props.overlay == Overlay::RunConfig, |anchor| {
                        anchor.child(
                            deferred(
                                anchored()
                                    .snap_to_window_with_margin(px(8.0))
                                    .child(self.run_config_dropdown()),
                            )
                            .with_priority(1),
                        )
                    }),
            )
            .child(Self::icon(Icon::PLAY, 15.0, Palette::muted()))
            .child(Self::icon(Icon::BUG, 15.0, Palette::muted()))
            // Equal spacers either side centre the search box in the toolbar.
            .child(div().flex_1())
            .child(
                Self::clickable("search-everywhere", cx, |shell, _| {
                    shell.open_palette(Mode::Files)
                })
                    .flex_1()
                    .max_w(px(400.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(7.0))
                    .px(px(9.0))
                    .py(px(4.0))
                    .rounded(px(crate::tokens::px_of(crate::generated_tokens::RADIUS_SM)))
                    .border_1()
                    .border_color(rgb(Palette::divider()))
                    .child(Self::icon(Icon::MAGNIFYING_GLASS, 13.0, Palette::muted()))
                    .child(Self::label("Search everywhere", 11.5, Palette::muted()))
                    .child(Self::code("⌘⇧F", 10.5, Palette::dim())),
            )
            .child(div().flex_1())
            .child(Self::pill(Icon::CROSSHAIR, "Spec pins"))
            .child(Self::pill(Icon::DESKTOP, "Local"))
            .child(
                Self::clickable("density", cx, |shell, _| shell.cycle_density())
                    .child(Self::icon(Icon::SLIDERS_HORIZONTAL, 15.0, Palette::muted())),
            )
    }

    /// A5: a toolbar control the prototype labels rather than leaving as a bare
    /// glyph, so its meaning does not depend on recognising the icon.
    fn pill(glyph: &'static str, label: &str) -> impl IntoElement {
        div()
            .h(px(24.0))
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(10.0))
            .rounded(px(crate::tokens::px_of(crate::generated_tokens::RADIUS_SM)))
            .border_1()
            .border_color(rgb(Palette::divider()))
            .child(Self::icon(glyph, 12.0, Palette::muted()))
            .child(Self::label(label, 11.0, Palette::muted()))
    }

    /// A2: the frame-timing panel. The prototype gates it on the performance
    /// readout — `hud: s.hud || perf === 'hud'` — so it is a depicted state
    /// reached through a prop the shell already carries, not a new one.
    fn latency_hud(&self) -> impl IntoElement {
        let histogram = fixture::frame_histogram();
        let tallest = histogram.iter().copied().fold(f32::MIN, f32::max);

        div()
            .absolute()
            .right(px(12.0))
            .bottom(px(Chrome::status_bar() + 12.0))
            .w(px(310.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .p(px(12.0))
            .rounded(px(crate::tokens::px_of(crate::generated_tokens::RADIUS_MD)))
            .bg(rgb(Palette::surface()))
            .border_1()
            .border_color(rgb(Palette::divider()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .child(Self::label(fixture::LATENCY_TITLE, 12.0, Palette::text()))
                    .child(div().flex_1())
                    .child(Self::icon(Icon::X, 10.0, Palette::dim())),
            )
            .child(
                div()
                    .h(px(34.0))
                    .flex()
                    .items_end()
                    .gap(px(2.0))
                    .children(histogram.into_iter().map(|sample| {
                        // Every bar is drawn: a histogram that hid its tail would
                        // hide exactly the frames the budget is about.
                        div()
                            .w(px(6.0))
                            .h(px((sample / tallest * 34.0).max(2.0)))
                            .bg(rgb(Palette::syn_keyword()))
                    })),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(Self::code(fixture::LATENCY_P50, 10.5, Palette::text()))
                    .child(Self::code(fixture::LATENCY_P99, 10.5, Palette::text()))
                    .child(Self::code(fixture::LATENCY_BUDGET, 10.5, Palette::muted())),
            )
            .child(Self::label(fixture::LATENCY_NOTE, 10.5, Palette::dim()))
    }

    /// Rail: 44px wide, the tool window switcher.
    fn rail(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(Chrome::rail_width()))
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.0))
            .py(px(10.0))
            .bg(rgb(Palette::surface()))
            .children(Self::rail_items().into_iter().map(|(tab, glyph)| {
                let active = tab == self.props.rail_tab;
                div()
                    .id(SharedString::from(format!("rail-{tab:?}")))
                    .size(px(28.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .when(active, |item| item.bg(rgb(Palette::panel())))
                    // Selecting a rail destination that is already selected
                    // collapses the tool window, as the prototype does.
                    .on_click(cx.listener(move |shell, _event, _window, cx| {
                        shell.select_rail(tab);
                        cx.notify();
                    }))
                    .child(Self::icon(
                        glyph,
                        17.0,
                        if active { Palette::accent() } else { Palette::muted() },
                    ))
            }))
    }

    /// The rail, as the prototype declares it.
    fn rail_items() -> Vec<(RailTab, &'static str)> {
        vec![
            (RailTab::Project, Icon::FOLDER_OPEN),
            (RailTab::Structure, Icon::LIST_DASHES),
            (RailTab::Commit, Icon::GIT_BRANCH),
            (RailTab::Find, Icon::MAGNIFYING_GLASS),
            (RailTab::History, Icon::CLOCK_COUNTER_CLOCKWISE),
            (RailTab::Packs, Icon::PUZZLE_PIECE),
        ]
    }

    /// Which panel the tool window shows. Packs is a view rather than a panel:
    /// selecting it moves the rail highlight and replaces the editor, while the
    /// tool window keeps the panel it already had, as the prototype does with
    /// `activeRail = view === 'plugins' ? 'plugins' : tool`.
    fn tool_panel(tab: RailTab) -> RailTab {
        match tab {
            RailTab::Packs => RailTab::Project,
            other => other,
        }
    }

    fn panel_title(tab: RailTab) -> &'static str {
        match tab {
            RailTab::Project => "Project",
            RailTab::Structure => "Structure",
            RailTab::Commit => "Commit",
            RailTab::Find => "Find in project",
            RailTab::History => "Local history",
            RailTab::Packs => "Packs and extensions",
        }
    }

    /// Rows for the selected panel: depth, glyph, label, selected, state letter.
    fn panel_rows(tab: RailTab) -> Vec<(u8, &'static str, &'static str, bool, Option<Vcs>)> {
        match tab {
            RailTab::Project => Self::tree_rows(),
            RailTab::Structure => vec![
                (0, Icon::BRACKETS_CURLY, "Shell", true, None),
                (1, Icon::DATABASE, "props: Props", false, None),
                (1, Icon::DATABASE, "profile: DensityProfile", false, None),
                (1, Icon::FUNCTION, "new(props)", false, None),
                (1, Icon::FUNCTION, "toolbar()", false, None),
                (1, Icon::FUNCTION, "rail()", false, None),
                (1, Icon::FUNCTION, "tool_window()", false, None),
                (1, Icon::FUNCTION, "editor()", false, None),
                (1, Icon::FUNCTION, "dock(height)", false, None),
                (1, Icon::FUNCTION, "status_bar()", false, None),
                (0, Icon::BRACKETS_CURLY, "NoOpCommandSink", false, None),
                (1, Icon::FUNCTION, "dispatch(command)", false, None),
            ],
            RailTab::Commit => vec![
                (0, Icon::GIT_BRANCH, "Changes", true, None),
                (1, Icon::FILE_CODE, "shell.rs", false, Some(Vcs::Modified)),
                (1, Icon::FILE_CODE, "props.rs", false, Some(Vcs::Modified)),
                (1, Icon::FILE_CODE, "icons.rs", false, Some(Vcs::Added)),
                (1, Icon::FILE_TEXT, "Cargo.toml", false, Some(Vcs::Modified)),
                (0, Icon::FOLDER, "Unversioned", false, None),
                (1, Icon::FILE_CODE, "gate-fidelity", false, Some(Vcs::Added)),
            ],
            RailTab::Find => vec![
                (0, Icon::MAGNIFYING_GLASS, "Chrome::toolbar", true, None),
                (1, Icon::FILE_CODE, "shell.rs:59", false, None),
                (1, Icon::FILE_CODE, "layout.rs:8", false, None),
                (1, Icon::FILE_CODE, "tokens.rs:31", false, None),
                (0, Icon::MAGNIFYING_GLASS, "CHROME_TOOLBAR", false, None),
                (1, Icon::FILE_CODE, "generated_tokens.rs:14", false, None),
            ],
            RailTab::History => vec![
                (0, Icon::CLOCK_COUNTER_CLOCKWISE, "today 21:43", true, None),
                (1, Icon::PENCIL_SIMPLE, "shell.rs — dock tabs moved", false, None),
                (0, Icon::CLOCK_COUNTER_CLOCKWISE, "today 21:12", false, None),
                (1, Icon::PENCIL_SIMPLE, "shell.rs — icons added", false, None),
                (0, Icon::CLOCK_COUNTER_CLOCKWISE, "today 20:48", false, None),
                (1, Icon::PLUS, "icons.rs — created", false, None),
            ],
            RailTab::Packs => Vec::new(),
        }
    }


    /// Tool window: width follows density.
    fn tool_window(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(self.tool_window_width()))
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(Palette::panel()))
            .child(
                div()
                    .h(px(Chrome::dock_header()))
                    .flex()
                    .items_center()
                    .px(px(10.0))
                    .gap(px(8.0))
                    .child(Self::section_label(
                        Self::panel_title(Self::tool_panel(self.props.rail_tab)),
                        Palette::muted(),
                    ))
                    .child(div().flex_1())
                    .when(Self::tool_panel(self.props.rail_tab) == RailTab::Project, |header| {
                        header.child(Self::label(fixture::FILE_COUNT, 10.5, Palette::dim()))
                    })
                    .child(
                        Self::clickable("tool-collapse", cx, |shell, _| {
                            shell.collapse_tool_window(true)
                        })
                        .child(Self::icon(Icon::CARET_DOUBLE_LEFT, 12.0, Palette::dim())),
                    )
                    .child(
                        Self::clickable("tool-side", cx, |shell, _| shell.toggle_tool_side())
                        .child(Self::icon(Icon::SIDEBAR_SIMPLE, 12.0, Palette::dim())),
                    ),
            )
            .children(Self::panel_rows(Self::tool_panel(self.props.rail_tab)).into_iter().map(|row| {
                let (depth, glyph, name, selected, status) = row;
                // A file's colour follows its version-control state, as the
                // prototype's warning colour is documented for modified marks.
                let name_colour = match status {
                    Some(Vcs::Added) => Palette::success(),
                    Some(Vcs::Modified) => Palette::warning(),
                    None if selected => Palette::text(),
                    None => Palette::muted(),
                };
                div()
                    .h(px(self.profile.row_height))
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .pl(px(10.0 + depth as f32 * 12.0))
                    .pr(px(10.0))
                    .when(selected, |row| row.bg(rgb(Palette::divider())))
                    .child(Self::icon(glyph, 13.0, if selected { Palette::accent() } else { Palette::dim() }))
                    .child(Self::label(name, self.profile.ui_font_size, name_colour))
                    .child(div().flex_1())
                    .when_some(status, |row, state| {
                        row.child(
                            div()
                                .font_family(fonts::UI_MEDIUM_FAMILY)
                                .text_size(px(10.5))
                                .text_color(rgb(match state {
                                    Vcs::Added => Palette::success(),
                                    Vcs::Modified => Palette::warning(),
                                }))
                                .child(SharedString::from(match state {
                                    Vcs::Added => "A",
                                    Vcs::Modified => "M",
                                })),
                        )
                    })
            }))
    }

    /// The project tree the prototype composes: depth, glyph, label, selection,
    /// and version-control state where the file has one.
    fn tree_rows() -> Vec<(u8, &'static str, &'static str, bool, Option<Vcs>)> {
        fixture::tree()
            .into_iter()
            .map(|row| {
                let vcs = row.vcs.map(|marker| match marker {
                    fixture::Vcs::Modified => Vcs::Modified,
                    fixture::Vcs::Added => Vcs::Added,
                });
                (row.depth, row.glyph, row.name, row.name == fixture::SELECTED_FILE, vcs)
            })
            .collect()
    }

    /// Editor: tab strip, breadcrumbs, then the buffer surface.
    fn editor(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(Palette::bg()))
            .child(
                div()
                    .h(px(Chrome::tab_strip()))
                    .flex()
                    .items_center()
                    .gap(px(1.0))
                    .bg(rgb(Palette::surface()))
                    .children(fixture::tabs().into_iter().enumerate().map(|(index, tab)| {
                        let active = index == self.props.active_tab;
                        Self::clickable(SharedString::from(format!("tab-{index}")), cx, move |shell, _| {
                            shell.select_tab(index)
                        })
                            .h_full()
                            .flex()
                            .items_center()
                            .px(px(14.0))
                            .bg(rgb(if active { Palette::bg() } else { Palette::surface() }))
                            .gap(px(7.0))
                            .child(Self::icon(tab.glyph, 12.0, tab.tint.colour()))
                            .child(Self::label(
                                tab.name,
                                12.0,
                                if active { Palette::text() } else { Palette::muted() },
                            ))
                            // A dirty buffer shows a dot where a clean one shows
                            // its close control.
                            .child(if tab.dirty {
                                div()
                                    .w(px(6.0))
                                    .h(px(6.0))
                                    .rounded(px(3.0))
                                    .bg(rgb(Palette::accent()))
                                    .into_any_element()
                            } else {
                                Self::icon(Icon::X, 9.0, Palette::dim()).into_any_element()
                            })
                    }))
                    .child(div().flex_1())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.0))
                            .pr(px(12.0))
                            .child(Self::icon(Icon::GIT_BRANCH, 14.0, Palette::muted()))
                            .child(Self::icon(Icon::COLUMNS, 14.0, Palette::muted()))
                            .child(Self::icon(Icon::TREE_STRUCTURE, 14.0, Palette::dim())),
                    ),
            )
            .child(
                div()
                    .h(px(Chrome::breadcrumbs()))
                    .flex()
                    .items_center()
                    .px(px(14.0))
                    .bg(rgb(Palette::bg()))
                    .gap(px(6.0))
                    .children(fixture::breadcrumbs().into_iter().enumerate().flat_map(
                        |(index, crumb)| {
                            let last = index == 4;
                            [
                                Self::icon(
                                    match index {
                                        0 => Icon::FOLDER,
                                        1 => Icon::PACKAGE,
                                        4 => Icon::FUNCTION,
                                        _ => Icon::FILE_CODE,
                                    },
                                    11.0,
                                    Palette::dim(),
                                )
                                .into_any_element(),
                                Self::label(
                                    crumb,
                                    10.5,
                                    if last { Palette::muted() } else { Palette::dim() },
                                )
                                .into_any_element(),
                                if last {
                                    div().into_any_element()
                                } else {
                                    Self::icon(Icon::CARET_RIGHT, 9.0, Palette::dim())
                                        .into_any_element()
                                },
                            ]
                        },
                    ))
                    .child(div().flex_1())
                    .child(Self::label(fixture::BREADCRUMB_NOTE, 10.5, Palette::dim())),
            )
            // A1: the prototype puts the indexing banner inside the editor,
            // directly below the breadcrumbs, not above the status bar.
            .child(self.indexing_banner())
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .py(px(8.0))
                    .children(fixture::buffer().into_iter().map(|line| {
                        div()
                            .h(px(self.profile.code_line_height))
                            .flex()
                            .items_center()
                            // A6: the prototype bands the caret's line.
                            .when(line.current, |row| row.bg(rgb(Palette::panel())))
                            .child(
                                div()
                                    .w(px(52.0))
                                    .flex()
                                    .items_center()
                                    .justify_end()
                                    .pr(px(12.0))
                                    .gap(px(6.0))
                                    // A6: a breakpoint sits in the gutter, left of
                                    // the number.
                                    .when(line.breakpoint, |gutter| {
                                        gutter.child(
                                            div()
                                                .w(px(7.0))
                                                .h(px(7.0))
                                                .rounded(px(4.0))
                                                .bg(rgb(Palette::error())),
                                        )
                                    })
                                    .child(Self::code(
                                        &format!("{}", line.number),
                                        self.profile.code_font_size,
                                        Palette::line_number(),
                                    )),
                            )
                            // A6: a 3px change bar marks an edited line.
                            .child(
                                div().w(px(3.0)).h_full().when(line.changed, |bar| {
                                    bar.bg(rgb(Palette::syn_keyword()))
                                }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .pl(px(12.0))
                                    .children(line.spans.into_iter().map(|(syntax, text)| {
                                        Self::code(text, self.profile.code_font_size, syntax.colour())
                                    })),
                            )
                    })),
            )
            // The palette replaces the popup rather than stacking over it.
            .when(self.props.completion_open && self.props.overlay != Overlay::Palette, |editor| {
                editor.child(self.completion())
            })
    }






    /// Status bar: 26px, with the performance readout when the property asks.
    fn status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(Chrome::status_bar()))
            .w_full()
            .flex()
            .items_center()
            .gap(px(16.0))
            .px(px(12.0))
            .bg(rgb(Palette::surface()))
            // B9: the prototype's host readout is the machine the session runs
            // on, which in the depicted state is local.
            .child(Self::icon(Icon::DESKTOP, 12.0, Palette::dim()))
            .child(Self::label(fixture::HOST, 10.5, Palette::muted()))
            .child(Self::icon(Icon::GIT_BRANCH, 12.0, Palette::dim()))
            .child(Self::label(fixture::BRANCH, 10.5, Palette::muted()))
            .child(Self::code(fixture::AHEAD, 10.5, Palette::syn_type()))
            .child(Self::code(fixture::BEHIND, 10.5, Palette::warning()))
            .child(Self::progress(52.0))
            .child(Self::label(
                &format!("{} · nothing disabled", fixture::INDEXING_LABEL),
                10.5,
                Palette::syn_type(),
            ))
            .child(div().flex_1())
            .children(fixture::servers().into_iter().map(|(name, size)| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0)).bg(rgb(Palette::success())))
                    .child(Self::label(&format!("{name} {size}"), 10.5, Palette::muted()))
            }))
            .when(self.props.perf_readout != PerfReadout::Off, |bar| {
                bar.child(
                    Self::clickable("perf-readout", cx, |shell, _| shell.toggle_hud())
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(Self::icon(Icon::GAUGE, 12.0, Palette::dim()))
                        .child(Self::code(fixture::STATUS_LATENCY, 10.5, Palette::accent())),
                )
            })
            .child(Self::code(fixture::CARET, 10.5, Palette::muted()))
            .children(
                fixture::ENCODING
                    .into_iter()
                    .map(|item| Self::label(item, 10.5, Palette::muted())),
            )
    }

    /// Correction 8: the indexing strip. Present while the index builds, and
    /// explicit that nothing is blocked while it does.
    /// A1: a full-width row inside the editor, below the breadcrumbs. The
    /// prototype puts it there rather than at the window's bottom edge, where an
    /// earlier revision of this shell had it.
    fn indexing_banner(&self) -> impl IntoElement {
        div()
            .h(px(26.0))
            .w_full()
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(12.0))
            .bg(rgb(Palette::panel()))
            .child(Self::icon(Icon::DATABASE, 12.0, Palette::accent()))
            .child(Self::label(fixture::INDEXING_BANNER, 11.5, Palette::text()))
            .child(div().flex_1())
            .child(Self::progress(90.0))
    }

    /// The prototype's indexing track: a rounded rail with an accent fill.
    fn progress(width: f32) -> impl IntoElement {
        div()
            .w(px(width))
            .h(px(3.0))
            .rounded(px(2.0))
            .bg(rgb(Palette::divider()))
            .child(
                div()
                    .w(relative(fixture::INDEXING))
                    .h(px(3.0))
                    .rounded(px(2.0))
                    .bg(rgb(Palette::accent())),
            )
    }



    /// Correction 7: the modal has four parts — title with its source and
    /// timing, a search input, the result list, and a footer of key hints.
    fn palette(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let all = self.props.palette_mode.rows();
        let shown = matching(all.clone(), &self.props.palette_query);

        div()
            .absolute()
            .inset_0()
            .flex()
            .justify_center()
            .items_start()
            .bg(rgba(0x0d0f189e))
            .child(
                div()
                    .mt(px(82.0))
                    .w(relative(0.88))
                    .max_w(px(680.0))
                    .flex()
                    .flex_col()
                    .rounded(px(crate::tokens::px_of(crate::generated_tokens::RADIUS_LG)))
                    .bg(rgb(Palette::surface()))
                    .border_1()
                    .border_color(rgb(Palette::muted()))
                    // 1. Title: the scopes the search runs against.
                    .child(
                        div()
                            .h(px(34.0))
                            .flex()
                            .items_center()
                            .px(px(6.0))
                            .children(
                                [Mode::Files, Mode::Commands, Mode::Structural]
                                    .into_iter()
                                    .map(|mode| (mode, mode.glyph(), mode.label(), mode == self.props.palette_mode))
                                .map(|(mode, glyph, name, active)| {
                                    Self::clickable(
                                        SharedString::from(format!("palette-{mode:?}")),
                                        cx,
                                        move |shell, _| shell.open_palette(mode),
                                    )
                                        .h(px(26.0))
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .px(px(10.0))
                                        .rounded(px(crate::tokens::px_of(
                                            crate::generated_tokens::RADIUS_SM,
                                        )))
                                        .when(active, |tab| tab.bg(rgb(Palette::panel())))
                                        .child(Self::icon(
                                            glyph,
                                            12.0,
                                            if active { Palette::accent() } else { Palette::dim() },
                                        ))
                                        .child(Self::label(
                                            name,
                                            11.5,
                                            if active { Palette::text() } else { Palette::muted() },
                                        ))
                                }),
                            )
                            .child(div().flex_1())
                            .child(
                                Self::clickable("palette-close", cx, |shell, _| shell.close_overlay())
                                .child(Self::icon(Icon::X, 10.0, Palette::dim())),
                            ),
                    )
                    // 2. Search input.
                    .child(
                        div()
                            .h(px(44.0))
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .px(px(14.0))
                            .bg(rgb(Palette::bg()))
                            .child(Self::icon(Icon::COMMAND, 14.0, Palette::accent()))
                            .child(Self::label(&self.props.palette_query, 14.0, Palette::text()))
                            .child(div().w(px(1.0)).h(px(16.0)).bg(rgb(Palette::accent())))
                            .child(div().flex_1())
                            .child(Self::label(&count_label(shown.len(), all.len()), 10.5, Palette::dim())),
                    )
                    // 3. Results.
                    .children(shown.into_iter().enumerate().map(|(index, row)| {
                        let selected = index == 0;
                        div()
                            .h(px(self.profile.row_height + 4.0))
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .px(px(14.0))
                            .when(selected, |entry| entry.bg(rgb(Palette::panel())))
                            .child(Self::icon(row.glyph, 13.0, Self::tint(row.tint)))
                            .child(Self::code(
                                row.name,
                                12.5,
                                if selected { Palette::text() } else { Palette::muted() },
                            ))
                            .child(div().flex_1())
                            .child(Self::code(row.detail, 10.5, Palette::dim()))
                            .child(Self::label(row.shortcut, 10.5, Palette::dim()))
                    }))
                    // 4. Footer: key hints and where the results came from.
                    .child(
                        div()
                            .h(px(28.0))
                            .flex()
                            .items_center()
                            .gap(px(14.0))
                            .px(px(14.0))
                            .bg(rgb(Palette::panel()))
                            .child(Self::key_hint(Icon::ARROW_DOWN, "insert"))
                            .child(Self::key_hint(Icon::ARROW_LINE_UP, "replace"))
                            .child(Self::key_hint(Icon::KEYBOARD, "space"))
                            .child(div().flex_1())
                            .child(Self::label("jdtls + sql pack, merged by source", 10.5, Palette::dim())),
                    ),
            )
    }

    /// The prototype tints a palette glyph by what the row is, not by where it
    /// sits in the list.
    pub(crate) fn tint(tint: Tint) -> u32 {
        match tint {
            Tint::Accent => Palette::accent(),
            Tint::Go => rgb_of(crate::generated_tokens::MANIFEST_GO_FILE_TINT_8FB3A5),
            Tint::Warning => Palette::warning(),
            Tint::Modified => Palette::warning(),
        }
    }

    /// Wraps a control so it reacts. Every interactive element in the shell goes
    /// through here, so the cursor affordance and the identifier convention are
    /// the same everywhere and a control cannot be made clickable by accident.
    fn clickable(
        id: impl Into<SharedString>,
        cx: &mut Context<Self>,
        action: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> gpui::Stateful<gpui::Div> {
        div()
            .id(id.into())
            .cursor_pointer()
            .on_click(cx.listener(move |shell, _event, _window, cx| {
                action(shell, cx);
                cx.notify();
            }))
    }

    pub(crate) fn key_hint(glyph: &'static str, label: &str) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(5.0))
            .child(
                div()
                    .px(px(5.0))
                    .rounded(px(crate::tokens::px_of(crate::generated_tokens::RADIUS_SM)))
                    .bg(rgb(Palette::divider()))
                    .child(Self::icon(glyph, 10.0, Palette::muted())),
            )
            .child(Self::label(label, 10.5, Palette::muted()))
    }

    /// Completion popup: the same four parts as the palette — title with its
    /// source and timing, the query, the results, and a footer of key hints.
    /// Anchored near the caret rather than centred.
    fn completion(&self) -> impl IntoElement {
        div()
            .absolute()
            .left(px(170.0))
            .bottom(px(10.0))
            .w(px(self.props.completion_style.width()))
            .flex()
            .flex_col()
            .rounded(px(crate::tokens::px_of(crate::generated_tokens::RADIUS_MD)))
            .bg(rgb(Palette::surface()))
            .border_1()
            .border_color(rgb(Palette::dim()))
            // 1. Title: which server answered, and how fast.
            .child(
                div()
                    .h(px(26.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(12.0))
                    .child(div().size(px(5.0)).rounded(px(3.0)).bg(rgb(Palette::success())))
                    .child(Self::code(fixture::COMPLETION_SOURCE, 10.5, Palette::text()))
                    .child(Self::code(fixture::COMPLETION_TIMING, 10.5, Palette::dim())),
            )
            // 2. The query being completed.
            .child(
                div()
                    .h(px(30.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(12.0))
                    .bg(rgb(Palette::bg()))
                    .child(Self::icon(Icon::MAGNIFYING_GLASS, 11.0, Palette::dim()))
                    .child(Self::label(fixture::COMPLETION_QUERY, 11.5, Palette::dim())),
            )
            // 3. Results, beside the documentation for the selected one.
            .child(
                div()
                    .flex()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .children(fixture::completions().into_iter().enumerate().map(
                                |(index, entry)| {
                                    let selected = index == 0;
                                    div()
                                        .h(px(self.profile.row_height))
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .px(px(12.0))
                                        .when(selected, |row| row.bg(rgb(Palette::panel())))
                                        .child(Self::code(
                                            entry.kind,
                                            10.0,
                                            if entry.certain {
                                                Palette::syn_type()
                                            } else {
                                                Palette::dim()
                                            },
                                        ))
                                        .child(Self::code(
                                            entry.name,
                                            12.0,
                                            if selected { Palette::text() } else { Palette::muted() },
                                        ))
                                        .child(Self::code(
                                            entry.signature,
                                            10.5,
                                            Palette::dim(),
                                        ))
                                        .child(div().flex_1())
                                        .child(Self::code(entry.source, 10.0, Palette::dim()))
                                },
                            )),
                    )
                    // A3: the documentation panel, which the prototype shows
                    // beside the list in the detail style.
                    .when(
                        self.props.completion_style == CompletionStyle::Detail,
                        |row| {
                            row.child(
                                div()
                                    .w(px(258.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(7.0))
                                    .px(px(11.0))
                                    .py(px(8.0))
                                    .bg(rgb(Palette::bg()))
                                    .children(fixture::DOC_SIGNATURE.into_iter().map(|line| {
                                        Self::code(line, 11.5, Palette::syn_type())
                                    }))
                                    .children(fixture::DOC_BODY.into_iter().map(|line| {
                                        Self::label(line, 11.5, Palette::muted())
                                    })),
                            )
                        },
                    ),
            )
            // 4. Footer: key hints and where the results came from.
            .child(
                div()
                    .h(px(26.0))
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .px(px(12.0))
                    .bg(rgb(Palette::panel()))
                    .child(Self::label("\u{23ce} insert", 10.5, Palette::muted()))
                    .child(Self::label("\u{21e5} replace", 10.5, Palette::muted()))
                    .child(div().flex_1())
                    .child(Self::label(fixture::COMPLETION_FOOTER, 10.0, Palette::dim())),
            )
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // T117. Two figures come out of here, and they are different things.
        //
        // The whole of this function is work on the UI thread, so its duration
        // is what Principle VI calls the longest task on that thread. If an
        // input is waiting to be painted, the time since it arrived is what the
        // user experienced as keystroke to paint. Both are recorded from the
        // path the product actually runs, not from a loop that imitates it.
        let began = Instant::now();
        let awaiting_paint = self.pending_input.lock().ok().and_then(|mut slot| slot.take());

        let viewport_height: f32 = window.viewport_size().height.into();
        self.commands.dispatch("shell.render");

        let side_first = self.props.tool_side == ToolSide::Left;

        let tree = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(Palette::bg()))
            .text_color(rgb(Palette::text()))
            .child(self.toolbar(cx))
            .child(
                // The tool window runs the full height of the window, and the
                // dock sits beside it rather than under it, so the terminal
                // never spans the panel.
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .when(side_first, |row| {
                        row.child(self.rail(cx)).when(!self.props.side_collapsed, |row| {
                            row.child(self.tool_window(cx)).child(Self::resize_handle(
                                "tool-edge",
                                true,
                                cx,
                                |shell, at| shell.place_tool_window_edge(at),
                            ))
                        })
                    })
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .flex()
                            .flex_col()
                            .when(self.props.rail_tab == RailTab::Packs, |column| {
                                column.child(self.packs_view())
                            })
                            .when(self.props.rail_tab != RailTab::Packs, |column| {
                                column.child(self.editor(cx))
                            })
                            .when(!self.props.dock_collapsed, |column| {
                                column
                                    .child(Self::resize_handle(
                                        "dock-edge",
                                        false,
                                        cx,
                                        move |shell, at| shell.place_dock_edge(at, viewport_height),
                                    ))
                                    .child(self.dock(viewport_height, cx))
                            })
                            .when(self.props.dock_collapsed, |column| {
                                column.child(self.collapsed_dock(cx))
                            }),
                    )
                    .when(!side_first, |row| {
                        row.when(!self.props.side_collapsed, |row| {
                            row.child(Self::resize_handle("tool-edge", true, cx, |shell, at| {
                                shell.place_tool_window_edge(at)
                            }))
                            .child(self.tool_window(cx))
                        })
                        .child(self.rail(cx))
                    }),
            )
            .child(self.status_bar(cx))
            .when(self.props.perf_readout == PerfReadout::Hud, |shell| {
                shell.child(self.latency_hud())
            })
            .when(self.props.overlay == Overlay::Palette, |shell| shell.child(self.palette(cx)));

        self.recorder.mark_first_frame();
        if let Some(arrived) = awaiting_paint {
            self.recorder.observe(Metric::KeystrokeToPaint, arrived.elapsed());
        }
        self.recorder.observe(Metric::LongestUiThreadTask, began.elapsed());
        tree
    }
}
