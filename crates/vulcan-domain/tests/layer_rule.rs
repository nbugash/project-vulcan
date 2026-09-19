//! T022: the layering rule.

use vulcan_domain::layer_edge::{Layer, LayerEdge, LayerRule};

fn edge(from_layer: Layer, to_layer: Layer) -> LayerEdge {
    LayerEdge {
        from: "a".into(),
        to: "b".into(),
        from_layer,
        to_layer,
        declared_at: "Cargo.toml:7".into(),
    }
}

#[test]
fn inward_edges_are_permitted() {
    assert!(LayerRule::is_permitted(&edge(Layer::Application, Layer::Domain)));
    assert!(LayerRule::is_permitted(&edge(Layer::Adapters, Layer::Application)));
    assert!(LayerRule::is_permitted(&edge(Layer::Composition, Layer::Adapters)));
}

#[test]
fn outward_edges_are_rejected() {
    assert!(!LayerRule::is_permitted(&edge(Layer::Domain, Layer::Adapters)));
    assert!(!LayerRule::is_permitted(&edge(Layer::Application, Layer::Composition)));
}

#[test]
fn same_layer_edges_are_permitted() {
    assert!(LayerRule::is_permitted(&edge(Layer::Adapters, Layer::Adapters)));
}

#[test]
fn an_undeclared_layer_does_not_parse() {
    assert!(Layer::parse("Presentation").is_none());
    assert!(Layer::parse("").is_none());
    assert!(Layer::parse("Domain").is_some());
}

#[test]
fn a_violation_names_both_crates_and_the_manifest_location() {
    let violation = LayerRule::describe_violation(&edge(Layer::Domain, Layer::Adapters));
    assert!(violation.contains("Domain"));
    assert!(violation.contains("Adapters"));
    assert!(violation.contains("Cargo.toml:7"));
}
