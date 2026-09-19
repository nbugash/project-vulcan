//! Reading extracted design values.
//!
//! Every dimension and colour the shell draws comes from `generated_tokens`,
//! which `gate-fidelity extract` writes from the prototype. Nothing here invents
//! a value; these helpers only convert the extracted strings into the types the
//! renderer wants.

use crate::generated_tokens as token;

/// `"46px"` -> `46.0`. A token that is not a pixel length is a programming
/// error rather than a runtime condition, so this panics with the value.
pub fn px_of(value: &str) -> f32 {
    value
        .trim()
        .strip_suffix("px")
        .unwrap_or_else(|| panic!("token {value:?} is not a pixel length"))
        .parse()
        .unwrap_or_else(|_| panic!("token {value:?} is not a number"))
}

/// `"#9184d9"` -> `0x9184d9`.
pub fn rgb_of(value: &str) -> u32 {
    let hex = value.trim().trim_start_matches('#');
    u32::from_str_radix(&hex[..6.min(hex.len())], 16)
        .unwrap_or_else(|_| panic!("token {value:?} is not a colour"))
}

pub struct Chrome;

impl Chrome {
    pub fn toolbar() -> f32 { px_of(token::CHROME_TOOLBAR) }
    pub fn remote_banner() -> f32 { px_of(token::CHROME_REMOTE_BANNER) }
    pub fn tab_strip() -> f32 { px_of(token::CHROME_TAB_STRIP) }
    pub fn breadcrumbs() -> f32 { px_of(token::CHROME_BREADCRUMBS) }
    pub fn dock_header() -> f32 { px_of(token::CHROME_DOCK_HEADER) }
    pub fn dock_tab_strip() -> f32 { px_of(token::CHROME_DOCK_TAB_STRIP) }
    pub fn status_bar() -> f32 { px_of(token::CHROME_STATUS_BAR) }
    pub fn rail_width() -> f32 { px_of(token::CHROME_RAIL_WIDTH) }
}

pub struct Palette;

impl Palette {
    pub fn bg() -> u32 { rgb_of(token::COLOR_BG) }
    pub fn surface() -> u32 { rgb_of(token::COLOR_SURFACE) }
    pub fn panel() -> u32 { rgb_of(token::COLOR_NEUTRAL_900) }
    pub fn text() -> u32 { rgb_of(token::COLOR_TEXT) }
    pub fn accent() -> u32 { rgb_of(token::COLOR_ACCENT) }
    pub fn muted() -> u32 { rgb_of(token::COLOR_NEUTRAL_500) }
    pub fn dim() -> u32 { rgb_of(token::COLOR_NEUTRAL_600) }
    pub fn divider() -> u32 { rgb_of(token::COLOR_NEUTRAL_800) }

    // The syntax palette. The manifest names each of these for its role, so
    // they are read from those entries rather than approximated from the
    // neutral and accent ramps, which is what an earlier revision did and got
    // visibly wrong: keywords resolved to accent-400 where the prototype paints
    // #b5abfc.
    pub fn syn_keyword() -> u32 { rgb_of(token::MANIFEST_KEYWORDS_ANNOTATIONS_TOML_B5ABFC) }
    pub fn syn_type() -> u32 { rgb_of(token::MANIFEST_TYPES_CLASSES_DOC_D2CEFD) }
    pub fn syn_function() -> u32 { rgb_of(token::MANIFEST_FUNCTION_AND_METHOD_E4E7F5) }
    pub fn syn_identifier() -> u32 { rgb_of(token::MANIFEST_IDENTIFIERS_NUMBERS_TERMINAL_CFD3E5) }
    pub fn syn_string() -> u32 { rgb_of(token::MANIFEST_STRING_LITERALS_B2B6CA) }
    pub fn syn_punctuation() -> u32 { rgb_of(token::MANIFEST_PUNCTUATION_OPERATORS_9397AB) }
    pub fn syn_comment() -> u32 { rgb_of(token::MANIFEST_COMMENTS_75798C) }
    pub fn line_number() -> u32 { rgb_of(token::MANIFEST_LINE_NUMBERS_595D6C) }

    /// Inlay hints: virtual text the language server supplies, not in the file.
    pub fn inlay() -> u32 { rgb_of(token::MANIFEST_INLAY_HINTS_VIRTUAL_E6E6EA) }

    /// The tint the prototype gives Go files in the tree and tab strip.
    pub fn go_tint() -> u32 { rgb_of(token::MANIFEST_GO_FILE_TINT_8FB3A5) }

    // The five semantic colours the manifest adds outside the design system,
    // because Nocturne is monochrome and state needs colour.
    pub fn error() -> u32 { rgb_of(token::MANIFEST_ERROR_DIAGNOSTICS_BREAKPOINT_D4736A) }
    pub fn warning() -> u32 { rgb_of(token::MANIFEST_WARNING_MODIFIED_FILE_C9A96A) }
    pub fn success() -> u32 { rgb_of(token::MANIFEST_SUCCESS_TESTS_PASSED_7FA98F) }
}
