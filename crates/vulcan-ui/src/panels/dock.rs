//! The bottom dock: header, content, and the tab strip along its bottom
//! edge, which is where the prototype puts it rather than the top.

use gpui::prelude::*;
use gpui::{div, px, rgb, IntoElement};

use crate::icons::Icon;
use gpui::Context;

use crate::shell::Shell;
use crate::tokens::{Chrome, Palette};

impl Shell {
    /// Dock: content first, tab strip along the bottom edge as the prototype
    /// composes it. Capped so the editor always keeps room.
    pub(crate) fn dock(&self, viewport_height: f32, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(self.dock_height_for(viewport_height)))
            .w_full()
            .flex()
            .flex_col()
            .bg(rgb(Palette::panel()))
            .child(
                div()
                    .h(px(Chrome::dock_header()))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(12.0))
                    .child(Self::icon(Icon::TERMINAL_WINDOW, 12.0, Palette::accent()))
                    .child(Self::label(crate::fixture::DOCK_TITLE, 11.5, Palette::text()))
                    .child(Self::label(crate::fixture::DOCK_SUBTITLE, 10.5, Palette::dim()))
                    .child(div().flex_1())
                    .child(Self::icon(Icon::ARROW_LINE_DOWN, 12.0, Palette::dim()))
                    .child(Self::icon(Icon::X, 11.0, Palette::dim())),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .px(px(12.0))
                    .py(px(4.0))
                    .children(Self::terminal_lines().into_iter().map(|(text, colour)| {
                        div()
                            .h(px(self.profile.code_line_height))
                            .child(Self::code(text, self.profile.code_font_size, colour))
                    })),
            )
            .child(
                // Tabs along the bottom edge, not the top.
                div()
                    .h(px(Chrome::dock_tab_strip()))
                    .flex()
                    .items_center()
                    .pr(px(12.0))
                    .bg(rgb(Palette::surface()))
                    .children(crate::fixture::dock_tabs().into_iter().enumerate().map(
                        |(index, (glyph, label, badge))| {
                            let active = index == self.props().dock_panel;
                            Self::dock_tab_button(index, glyph, label, badge, active, cx)
                        },
                    ))
                    .child(div().flex_1())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(Self::icon(Icon::COMMAND, 12.0, Palette::dim()))
                            .child(Self::label("Commands", 11.5, Palette::muted())),
                    ),
            )
    }

    /// The selected tab is filled and its label goes to full-strength text, as
    /// the prototype computes it: `bg` becomes neutral-900 and `color` becomes
    /// `--color-text` while the dock is open on that tab.
    fn dock_tab_button(
        index: usize,
        glyph: &'static str,
        label: &str,
        badge: &str,
        active: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        Self::clickable_panel(
            gpui::SharedString::from(format!("dock-{index}")),
            cx,
            move |shell, _| shell.select_dock_panel(index),
        )
        .child(Self::dock_tab(glyph, label, badge, active))
    }

    fn dock_tab(glyph: &'static str, label: &str, badge: &str, active: bool) -> impl IntoElement {
        div()
            .h_full()
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(11.0))
            .when(active, |tab| tab.bg(rgb(Palette::panel())))
            .child(Self::icon(glyph, 13.0, if active { Palette::accent() } else { Palette::dim() }))
            .child(Self::label(
                label,
                11.5,
                if active { Palette::text() } else { Palette::muted() },
            ))
            .when(!badge.is_empty(), |tab| {
                tab.child(Self::label(
                    badge,
                    10.0,
                    if badge == "paused" { Palette::accent() } else { Palette::warning() },
                ))
            })
    }

    fn terminal_lines() -> Vec<(&'static str, u32)> {
        crate::fixture::terminal()
            .into_iter()
            .map(|(output, text)| (text, output.colour()))
            .collect()
    }

    /// Collapsed dock: the tab strip remains so the panels stay reachable, and
    /// no tab is highlighted because none is showing.
    pub(crate) fn collapsed_dock(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(Chrome::dock_tab_strip()))
            .w_full()
            .flex()
            .items_center()
            .pr(px(12.0))
            .bg(rgb(Palette::surface()))
            .children(crate::fixture::dock_tabs().into_iter().enumerate().map(
                |(index, (glyph, label, badge))| {
                    Self::dock_tab_button(index, glyph, label, badge, false, cx)
                },
            ))
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(Self::icon(Icon::COMMAND, 12.0, Palette::dim()))
                    .child(Self::label("Commands", 11.5, Palette::muted())),
            )
    }
}
