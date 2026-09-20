//! Adapter implementations. Everything that touches the filesystem, a process,
//! a renderer or a clock lives here, behind a port defined in vulcan-app.

pub mod manifest;
pub mod measurement;
pub mod rendering;
pub mod reporting;
pub mod tokens;
