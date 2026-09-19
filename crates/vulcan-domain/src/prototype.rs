//! What makes a document usable as the signed-off prototype.
//!
//! The prototype's filename is not a rule — it has already changed once, from
//! `Vulcan-IDE.html` to `Vulcan IDE (standalone).html`, and a hardcoded name
//! turned that rename into gates that read no prototype at all while still
//! reporting a pass. What is a rule is that the prototype fetches nothing
//! outside itself: a reference render that depends on files it does not contain
//! is not reproducible, and the exact comparison depends on reproducibility.

/// Whether the document fetches nothing outside itself.
///
/// Only *fetched* references count. `xmlns="http://www.w3.org/2000/svg"` is a
/// namespace identifier that is never resolved over the network; treating it as
/// an external dependency rejects every real candidate.
pub fn is_self_contained(text: &str) -> bool {
    !fetches_sibling(text) && !fetches_remote(text)
}

fn fetches_sibling(text: &str) -> bool {
    ["_ds/", "\"fonts/", "\"icons/", "'fonts/", "'icons/"]
        .iter()
        .any(|reference| text.contains(reference))
}

fn fetches_remote(text: &str) -> bool {
    ["src=", "href="].iter().any(|attribute| {
        text.match_indices(attribute).any(|(at, _)| {
            let value = text[at + attribute.len()..].trim_start();
            let value = value.trim_start_matches(['\\', '"', '\'']);
            value.starts_with("http://") || value.starts_with("https://")
        })
    })
}

#[cfg(test)]
mod tests {
    use super::is_self_contained;

    #[test]
    fn an_svg_namespace_is_not_an_external_dependency() {
        assert!(is_self_contained(r#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#));
    }

    #[test]
    fn a_fetched_stylesheet_is() {
        assert!(!is_self_contained(r#"<link href="https://cdn.example/x.css">"#));
    }

    #[test]
    fn a_sibling_asset_directory_is() {
        assert!(!is_self_contained(r#"<link href="_ds/nocturne/styles.css">"#));
    }

    #[test]
    fn an_inlined_document_is_self_contained() {
        assert!(is_self_contained("<style>body{font-family:'JetBrains Mono'}</style>"));
    }
}
