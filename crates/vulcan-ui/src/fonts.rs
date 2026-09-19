//! The prototype's typefaces, vendored and loaded from the repository.
//!
//! `mockups/` ships Inter, JetBrains Mono and Phosphor as woff2, which the text
//! shaper cannot read, so `assets/fonts/` holds the same glyphs converted to
//! TrueType. Nothing is fetched: the interface must render correctly with no
//! network (FR-032).

use std::borrow::Cow;

/// Inter, the prototype's interface typeface, at the weights it declares.
pub const INTER_400: &[u8] = include_bytes!("../../../assets/fonts/inter-latin-400.ttf");
pub const INTER_500: &[u8] = include_bytes!("../../../assets/fonts/inter-latin-500.ttf");
pub const INTER_600: &[u8] = include_bytes!("../../../assets/fonts/inter-latin-600.ttf");

/// JetBrains Mono, which the prototype sets all code in.
pub const CODE_400: &[u8] = include_bytes!("../../../assets/fonts/jetbrains-mono-400.ttf");
pub const CODE_500: &[u8] = include_bytes!("../../../assets/fonts/jetbrains-mono-500.ttf");

/// Phosphor, the icon font the prototype draws all 58 of its glyphs from.
pub const PHOSPHOR: &[u8] = include_bytes!("../../../assets/fonts/Phosphor.ttf");

/// Family names as the fonts declare them, for `.font_family(..)`.
pub const UI_FAMILY: &str = "Inter";
pub const UI_MEDIUM_FAMILY: &str = "Inter Medium";
pub const ICON_FAMILY: &str = "Phosphor";

/// The prototype's code stack is `'JetBrains Mono', ui-monospace, Menlo,
/// monospace`. The face is vendored, so the first entry always resolves and the
/// fallbacks never come into play — which is what makes the exact comparison
/// reproducible across platforms.
pub const CODE_FAMILY: &str = "JetBrains Mono";

pub fn embedded() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(INTER_400),
        Cow::Borrowed(INTER_500),
        Cow::Borrowed(INTER_600),
        Cow::Borrowed(CODE_400),
        Cow::Borrowed(CODE_500),
        Cow::Borrowed(PHOSPHOR),
    ]
}
