//! Every state the shell can reach, and what in the prototype depicts it.
//!
//! FR-033 requires that a state the prototype omits is recorded before it ships.
//! Enforcing that needs two things the code can check: the full list of states,
//! and, for each, a string that must occur in the prototype itself. Naming the
//! evidence is what stops the list becoming a set of claims about itself.

/// What in the signed-off prototype shows this state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depiction {
    /// A `data-screen-label` the prototype composes, or a state binding it
    /// carries. The text must appear verbatim in the prototype, whichever file
    /// under `mockups/` the adapter discovers as self-contained.
    Prototype(&'static str),
    /// The prototype does not depict it. The text is the heading under
    /// `## Prototype extensions` in the feature's design document.
    Extension(&'static str),
}

pub struct State {
    pub name: &'static str,
    pub depiction: Depiction,
}

pub fn reachable() -> Vec<State> {
    use Depiction::{Extension, Prototype};
    vec![
        state("window chrome", Prototype("data-screen-label=\\\"Chrome\\\"")),
        state("editor", Prototype("data-screen-label=\\\"Editor\\\"")),
        state("dock", Prototype("data-screen-label=\\\"Dock\\\"")),
        state("tool window", Prototype("data-screen-label=\\\"Tool window\\\"")),
        state("palette", Prototype("data-screen-label=\\\"Palette\\\"")),
        state("completion popup", Prototype("data-screen-label=\\\"Completion\\\"")),
        state("run configuration dropdown", Prototype("data-screen-label=\\\"Run configurations\\\"")),
        state("packs and extensions", Prototype("data-screen-label=\\\"Plugins\\\"")),
        state("rail: commit", Prototype("data-screen-label=\\\"Diff\\\"")),
        state("rail: find", Prototype("data-screen-label=\\\"Structural search\\\"")),
        state("rail: branches", Prototype("data-screen-label=\\\"Branches\\\"")),
        // The prototype binds this state even though it composes no screen for it.
        state("dock collapsed", Prototype("dockOpen:false")),
        state("density: compact, default, roomy", Prototype("density:p.density||'default'")),
        state("completion style: compact and detail", Prototype("compWidth:p.completionStyle")),
        // A2: depicted by the prototype, which composes it under a screen label
        // and gates it on the same performance-readout prop the shell carries.
        state("latency HUD", Prototype("data-screen-label=\\\"Latency HUD\\\"")),
        state("tool window collapsed", Extension("Tool window collapsed")),
        state("rail: local history", Extension("Local history tool window")),
    ]
}

fn state(name: &'static str, depiction: Depiction) -> State {
    State { name, depiction }
}
