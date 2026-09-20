//! Gate orchestration. Each use case coordinates ports and domain rules and
//! returns a verdict; none of them performs I/O directly.

pub mod check_boundaries;
pub mod compare_fidelity;
pub mod extract_tokens;
pub mod lint_off_token;
pub mod measure_budgets;
