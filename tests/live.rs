//! Live check against the public Roomvana API.
//!
//! Network test: run explicitly with `cargo test --test live -- --ignored`.

use roomvana::Catalog;

#[test]
#[ignore = "hits the live https://api.roomvana.co/options endpoint"]
fn live_catalog_parses_and_builds_a_design_link() {
    let catalog = Catalog::fetch().expect("live /options fetch should succeed");

    assert!(
        !catalog.room_slugs().is_empty(),
        "expected interior rooms, got none"
    );
    assert!(
        !catalog.style_slugs().is_empty(),
        "expected styles, got none"
    );
    assert!(
        catalog.style_slugs().iter().any(|style| style == "japandi"),
        "expected `japandi` among the live styles"
    );

    let link = catalog
        .design_link("kitchen", "japandi")
        .expect("kitchen x japandi should be a valid combination");
    assert_eq!(link, "https://roomvana.ai/design?room=kitchen&style=japandi");

    println!(
        "live catalog: {} rooms, {} exteriors, {} gardens, {} styles",
        catalog.room_slugs().len(),
        catalog.exterior_slugs().len(),
        catalog.garden_slugs().len(),
        catalog.style_slugs().len(),
    );
}
