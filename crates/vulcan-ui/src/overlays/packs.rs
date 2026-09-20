//! The language pack and extension view the rail's last tab selects.

use gpui::prelude::*;
use gpui::{div, px, rgb, IntoElement, SharedString};

use crate::fonts;
use crate::icons::Icon;
use crate::shell::Shell;
use crate::tokens::Palette;

impl Shell {
    /// Packs and extensions replaces the editor rather than filling a panel,
    /// as the prototype composes it.
    pub(crate) fn packs_view(&self) -> impl IntoElement {
        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .px(px(30.0))
            .py(px(24.0))
            .bg(rgb(Palette::bg()))
            .child(
                div()
                    .font_family(fonts::UI_MEDIUM_FAMILY)
                    .text_size(px(14.5))
                    .text_color(rgb(Palette::text()))
                    .child(SharedString::from("Packs and extensions")),
            )
            .child(Self::label(
                "Two kinds only. Declarative packs are manifests plus assets — grammars, queries, server and adapter definitions, themes, keymaps.",
                12.5,
                Palette::muted(),
            ))
            .children(
                [
                    (Icon::COFFEE, "Java pack", "Pack", "Grammar, queries, jdtls and the java debug adapter."),
                    (Icon::FILE_CODE, "Rust pack", "Pack", "rust-analyzer, CodeLLDB, cargo tasks."),
                    (Icon::PALETTE, "Nocturne", "Theme", "The design system this interface is built from."),
                    (Icon::TEXT_AA, "JetBrains Mono", "Font", "Pinned for deterministic code rendering."),
                ]
                .into_iter()
                .map(|(glyph, name, kind, detail)| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .px(px(14.0))
                        .py(px(10.0))
                        .rounded(px(crate::tokens::px_of(crate::generated_tokens::RADIUS_MD)))
                        .bg(rgb(Palette::surface()))
                        .child(Self::icon(glyph, 17.0, Palette::accent()))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(Self::label(name, 12.5, Palette::text()))
                                .child(Self::label(detail, 11.0, Palette::dim())),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .px(px(7.0))
                                .rounded(px(crate::tokens::px_of(crate::generated_tokens::RADIUS_SM)))
                                .bg(rgb(Palette::divider()))
                                .child(Self::label(kind, 10.0, Palette::muted())),
                        )
                }),
            )
    }
}
