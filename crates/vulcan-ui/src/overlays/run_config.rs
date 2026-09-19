//! The run configuration control and the dropdown it opens.

use gpui::prelude::*;
use gpui::{div, px, rgb, IntoElement};

use crate::icons::Icon;
use crate::shell::Shell;
use crate::tokens::Palette;

impl Shell {
    /// Correction 2: the run configuration control, and the dropdown it opens.
    pub(crate) fn run_config_button(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(7.0))
            .px(px(9.0))
            .py(px(4.0))
            .rounded(px(crate::tokens::px_of(crate::generated_tokens::RADIUS_SM)))
            .border_1()
            .border_color(rgb(Palette::divider()))
            .child(Self::icon(Icon::PLAY_CIRCLE, 13.0, Palette::accent()))
            .child(Self::label("PaymentsApp", 12.0, Palette::text()))
            .child(Self::icon(Icon::CARET_DOWN, 9.0, Palette::dim()))
    }

    pub(crate) fn run_config_dropdown(&self) -> impl IntoElement {
        div()
            // Sits directly beneath the control it belongs to.
            .mt(px(4.0))
            .w(px(300.0))
            .flex()
            .flex_col()
            .rounded(px(crate::tokens::px_of(crate::generated_tokens::RADIUS_MD)))
            .bg(rgb(Palette::surface()))
            .border_1()
            .border_color(rgb(Palette::dim()))
            .child(
                div()
                    .px(px(12.0))
                    .py(px(8.0))
                    .child(Self::section_label("Run configurations", Palette::dim())),
            )
            .children(
                [
                    (Icon::COFFEE, "PaymentsApp", "java · remote", true),
                    (Icon::TEST_TUBE, "OrderServiceTest", "java · test", false),
                    (Icon::SHIPPING_CONTAINER, "Run via Docker", "compose", false),
                    (Icon::TERMINAL_WINDOW, "cargo run -p shell-preview", "rust", false),
                ]
                .into_iter()
                .map(|(glyph, name, meta, active)| {
                    div()
                        .h(px(self.profile.row_height + 4.0))
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .px(px(12.0))
                        .when(active, |row| row.bg(rgb(Palette::panel())))
                        .child(Self::icon(glyph, 13.0, if active { Palette::accent() } else { Palette::dim() }))
                        .child(Self::label(name, 12.0, if active { Palette::text() } else { Palette::muted() }))
                        .child(div().flex_1())
                        .child(Self::code(meta, 10.5, Palette::dim()))
                }),
            )
            .child(
                div()
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(12.0))
                    .child(Self::icon(Icon::PENCIL_SIMPLE, 11.0, Palette::dim()))
                    .child(Self::label("Edit configurations…", 11.0, Palette::muted())),
            )
    }
}
