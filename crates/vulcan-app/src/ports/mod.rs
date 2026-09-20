//! Outbound ports. Every side effect this application performs is declared here
//! as a capability, never as a technology (Principle II).

pub mod command_sink;
pub mod constrained_runner;
pub mod image_compare;
pub mod render_capture;
pub mod token_source;
pub mod workspace_graph;
pub mod frame_recorder;
