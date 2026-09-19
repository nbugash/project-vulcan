use vulcan_domain::design_value::DesignValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenError {
    Unparseable { path: String, detail: String },
    MissingSource(String),
}

pub trait TokenSourcePort {
    /// Reads both sources: the stylesheet's custom properties and the values the
    /// asset manifest documents outside them.
    fn read_values(&self) -> Result<Vec<DesignValue>, TokenError>;

    /// Every dimension the prototype actually uses, named or not.
    ///
    /// A named token answers "what does the design system call this?". The lint
    /// asks a different question: "did this value come from the prototype at
    /// all?". A padding the prototype writes inline is legitimate without ever
    /// being named, and a width the reproduction invented is not legitimate
    /// however sensible it looks.
    fn read_vocabulary(&self) -> Result<Vec<String>, TokenError>;
}
