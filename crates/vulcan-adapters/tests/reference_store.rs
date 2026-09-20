//! T051: the reference store round-trips an image and its metadata.

use vulcan_adapters::rendering::reference_store::{ReferenceMetadata, ReferenceStore};
use vulcan_domain::rendering::{Image, Viewport};

fn scratch(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("vulcan-refs-{name}"));
    let _ = std::fs::remove_dir_all(&path);
    path
}

#[test]
fn an_image_round_trips_through_png() {
    let store = ReferenceStore::new(scratch("roundtrip"));
    let viewport = Viewport { width: 3, height: 2 };
    let image = Image { viewport, pixels: (0..24).map(|b| b as u8).collect() };
    let metadata = ReferenceMetadata {
        viewport,
        typefaces: vec!["Inter 400".into(), "Phosphor 2.1.1".into()],
        prototype_digest: "sha256:abc".into(),
        environment_digest: "sha256:def".into(),
        captured_at: "2026-09-19T00:00:00Z".into(),
    };

    store.save("shell", &image, &metadata).expect("saves");
    let (loaded_image, loaded_metadata) = store.load("shell").expect("loads");

    assert_eq!(loaded_image, image, "pixels must survive the round trip exactly");
    assert_eq!(loaded_metadata.prototype_digest, "sha256:abc");
    assert_eq!(loaded_metadata.environment_digest, "sha256:def");
    assert_eq!(loaded_metadata.typefaces.len(), 2);
}

#[test]
fn a_missing_reference_is_reported_rather_than_defaulted() {
    assert!(ReferenceStore::new(scratch("missing")).load("absent").is_err());
}
