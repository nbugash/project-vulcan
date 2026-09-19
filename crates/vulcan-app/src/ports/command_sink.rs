//! Where every shell control sends its activation.
//!
//! In F000 the only implementation does nothing. That is the point: a control
//! that partly acts cannot be told apart from a defect by the feature that later
//! owns the behaviour. F002 replaces the implementation, not this signature.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandId(pub &'static str);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchError {
    Unknown(String),
}

pub trait CommandSinkPort {
    fn dispatch(&self, command: CommandId) -> Result<(), DispatchError>;
}
