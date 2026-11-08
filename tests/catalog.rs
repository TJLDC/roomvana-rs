//! Unit tests against a fake catalog payload. No network access.

use roomvana::{Catalog, Error, SlugKind};

/// Shaped like the real `/options` response: the four arrays this crate reads,
/// plus keys it must ignore (surfaces, finishes, capability flags).
const FAKE_OPTIONS: &str = r#"{
    "rooms": ["kitchen", "living-room", "closet"],
    "exteriors": ["exterior"],
    "gardens": ["garden", "backyard"],
    "styles": ["modern", "japandi", "bohemian"],
    "surfaces": ["walls", "flooring"],
    "shades": {"walls": {"warm-white": {"brand": "Sherwin-Williams", "code": "SW 7008"}}},
    "custom_area": true,
    "floor_plan_mode": true,
    "costs": {"basic": 1, "hd": 3}
}"#;

fn catalog() -> Catalog {
    Catalog::from_json(FAKE_OPTIONS).expect("fake payload should parse")
}

#[test]
fn parses_the_four_taxonomy_arrays_and_ignores_unknown_keys() {
    let catalog = catalog();
    assert_eq!(catalog.room_slugs(), ["kitchen", "living-room", "closet"]);
    assert_eq!(catalog.exterior_slugs(), ["exterior"]);
    assert_eq!(catalog.garden_slugs(), ["garden", "backyard"]);
    assert_eq!(catalog.style_slugs(), ["modern", "japandi", "bohemian"]);
}

#[test]
fn rooms_and_styles_carry_slugs_and_display_names() {
    let catalog = catalog();

    let rooms = catalog.rooms();
    assert_eq!(rooms[0].slug, "kitchen");
    assert_eq!(rooms[0].name, "kitchen");
    assert_eq!(rooms[2].slug, "closet");
    assert_eq!(rooms[2].name, "walk-in closet");

    let styles = catalog.styles();
    assert_eq!(styles[1].slug, "japandi");
    assert_eq!(styles[1].name, "Japandi");
}

#[test]
fn spaces_cover_interior_exterior_and_garden_in_order() {
    let spaces = catalog().spaces();
    let slugs: Vec<&str> = spaces.iter().map(|room| room.slug.as_str()).collect();
    assert_eq!(
        slugs,
        ["kitchen", "living-room", "closet", "exterior", "garden", "backyard"]
    );
}

#[test]
fn design_link_builds_the_documented_url() {
    let catalog = catalog();
    assert_eq!(
        catalog.design_link("kitchen", "japandi").unwrap(),
        "https://roomvana.ai/design?room=kitchen&style=japandi"
    );
    // Garden and exterior ids are valid values of the same `room` parameter.
    assert_eq!(
        catalog.design_link("backyard", "bohemian").unwrap(),
        "https://roomvana.ai/design?room=backyard&style=bohemian"
    );
    assert_eq!(
        catalog.design_link("exterior", "modern").unwrap(),
        "https://roomvana.ai/design?room=exterior&style=modern"
    );
}

#[test]
fn unknown_room_is_a_distinct_error_with_a_suggestion() {
    let error = catalog().design_link("kitchn", "japandi").unwrap_err();

    match &error {
        Error::UnknownSlug {
            kind,
            value,
            suggestion,
            available,
        } => {
            assert_eq!(*kind, SlugKind::Room);
            assert_eq!(value, "kitchn");
            assert_eq!(suggestion.as_deref(), Some("kitchen"));
            assert!(available.contains(&"exterior".to_string()));
            assert!(available.contains(&"garden".to_string()));
        }
        other => panic!("expected UnknownSlug, got {other:?}"),
    }

    assert!(error.is_unknown_slug());
    assert_eq!(error.suggestion(), Some("kitchen"));
    let message = error.to_string();
    assert!(message.contains("unknown room `kitchn`"), "{message}");
    assert!(message.contains("did you mean `kitchen`"), "{message}");
    assert!(message.contains("available rooms:"), "{message}");
}

#[test]
fn unknown_style_lists_every_valid_style() {
    let error = catalog().design_link("kitchen", "steampunk").unwrap_err();

    match &error {
        Error::UnknownSlug {
            kind,
            value,
            suggestion,
            available,
        } => {
            assert_eq!(*kind, SlugKind::Style);
            assert_eq!(value, "steampunk");
            assert_eq!(*suggestion, None);
            assert_eq!(available, &["modern", "japandi", "bohemian"]);
        }
        other => panic!("expected UnknownSlug, got {other:?}"),
    }

    let message = error.to_string();
    assert!(message.contains("unknown style `steampunk`"), "{message}");
    assert!(!message.contains("did you mean"), "{message}");
    assert!(
        message.contains("available styles: modern, japandi, bohemian"),
        "{message}"
    );
}

#[test]
fn a_near_miss_style_gets_a_suggestion() {
    let error = catalog().design_link("kitchen", "japandee").unwrap_err();
    assert_eq!(error.suggestion(), Some("japandi"));
}

#[test]
fn empty_taxonomy_is_a_parse_error_not_an_empty_catalog() {
    let error = Catalog::from_json(r#"{"rooms": [], "styles": ["modern"]}"#).unwrap_err();
    assert!(matches!(error, Error::Parse(_)), "{error:?}");
    assert!(error.to_string().contains("no spaces"), "{error}");

    let error = Catalog::from_json(r#"{"rooms": ["kitchen"], "styles": []}"#).unwrap_err();
    assert!(matches!(error, Error::Parse(_)), "{error:?}");
    assert!(error.to_string().contains("no styles"), "{error}");
}

#[test]
fn malformed_json_is_a_parse_error() {
    let error = Catalog::from_json("not json").unwrap_err();
    assert!(matches!(error, Error::Parse(_)), "{error:?}");
    assert!(!error.is_unknown_slug());
}

#[test]
fn blank_ids_from_the_server_are_dropped() {
    let catalog =
        Catalog::from_json(r#"{"rooms": ["kitchen", "", "  "], "styles": ["modern"]}"#).unwrap();
    assert_eq!(catalog.room_slugs(), ["kitchen"]);
}
