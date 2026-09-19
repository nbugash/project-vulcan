//! The application shell: an inbound adapter.
//!
//! Maps input to use cases and renders their output. GPUI types never leave this
//! crate, which is what keeps the framework choice contained at the edge
//! (Principle I) and what bounds the F002 retrofit when the shell is re-hosted
//! through plugin extension points.

pub mod fonts;
pub mod generated_tokens;
pub mod fixture;
pub mod icons;
pub mod props;
pub mod shell;
pub mod tokens;
pub mod palette;
pub mod overlays;
pub mod panels;
pub mod states;
