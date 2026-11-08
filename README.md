# roomvana

A small, synchronous Rust client for the public catalog of
[Roomvana](https://roomvana.ai) — an AI interior design tool. It reads the room
and style taxonomy from Roomvana's public API and builds pre-filled design
links.

- Website: <https://roomvana.ai>
- API: `GET https://api.roomvana.co/options` (public, no authentication)
- Documentation: <https://docs.rs/roomvana>

## What it does

1. `Catalog::fetch()` downloads `https://api.roomvana.co/options` once and
   parses it.
2. `catalog.rooms()`, `catalog.styles()` and friends return entries with both
   the slug (the id used in URLs) and a human-readable name.
3. `catalog.design_link(room, style)` validates both slugs and returns a link
   like `https://roomvana.ai/design?room=kitchen&style=japandi`.

## What it does not do

- It never generates images and never spends credits.
- It needs no API key, account or token.
- It only reads the catalog; there is no upload, billing or account surface.
- It does not cache responses. Fetch once and reuse the `Catalog`.

## Install

```toml
[dependencies]
roomvana = "0.1"
```

## Quick start

```rust
use roomvana::Catalog;

fn main() -> Result<(), roomvana::Error> {
    let catalog = Catalog::fetch()?;

    println!("{} rooms, {} styles", catalog.rooms().len(), catalog.styles().len());

    for style in catalog.styles() {
        println!("{} ({})", style.name, style.slug);
    }

    let link = catalog.design_link("living-room", "japandi")?;
    println!("{link}");
    // https://roomvana.ai/design?room=living-room&style=japandi

    Ok(())
}
```

`Catalog::fetch` is blocking, uses a 10 second timeout and no async runtime.
The catalog also covers exterior and garden spaces, which are valid values of
the same `room` parameter:

```rust
use roomvana::Catalog;

fn main() -> Result<(), roomvana::Error> {
    let catalog = Catalog::from_json(
        r#"{"rooms":["kitchen"],"exteriors":["exterior"],
            "gardens":["garden","backyard"],"styles":["modern"]}"#,
    )?;

    assert_eq!(catalog.spaces().len(), 4);
    assert_eq!(
        catalog.design_link("backyard", "modern")?,
        "https://roomvana.ai/design?room=backyard&style=modern"
    );
    Ok(())
}
```

`Catalog::from_json` accepts a payload you already have (a saved response, a
test fixture, or a proxy), which is also how the crate is tested without
network access.

## API

| Method | Returns |
| --- | --- |
| `Catalog::fetch()` | `Result<Catalog, Error>` — one GET to `/options` |
| `Catalog::from_json(&str)` | `Result<Catalog, Error>` — parse a payload in hand |
| `catalog.rooms()` | `Vec<Room>` — interior rooms |
| `catalog.exteriors()` | `Vec<Room>` — exterior spaces |
| `catalog.gardens()` | `Vec<Room>` — garden and outdoor spaces |
| `catalog.spaces()` | `Vec<Room>` — all of the above, in that order |
| `catalog.styles()` | `Vec<Style>` — design styles |
| `catalog.room_slugs()` / `style_slugs()` | `&[String]` — ids exactly as served |
| `catalog.contains_space(slug)` / `contains_style(slug)` | `bool` |
| `catalog.design_link(room, style)` | `Result<String, Error>` |

`Room` and `Style` each have `slug` and `name` fields.

## Errors

`roomvana::Error` distinguishes three cases:

- `Error::Network(String)` — DNS, TLS, timeout, connection reset or a non-2xx
  status.
- `Error::Parse(String)` — the body was not the expected JSON, or contained no
  styles or no spaces.
- `Error::UnknownSlug { kind, value, suggestion, available }` — one of the
  slugs you passed is not in the catalog. The message names the closest valid
  slug when the input looks like a typo, and always lists the valid values.
  `error.suggestion()` and `error.available()` expose the same data
  programmatically.

```rust
use roomvana::Catalog;

fn main() {
    let catalog = Catalog::from_json(
        r#"{"rooms":["kitchen"],"styles":["japandi"]}"#,
    )
    .unwrap();

    let error = catalog.design_link("kitchn", "japandi").unwrap_err();
    assert_eq!(error.suggestion(), Some("kitchen"));
    println!("{error}");
    // unknown room `kitchn` (did you mean `kitchen`?); available rooms: kitchen
}
```

## About display names

`GET /options` serves **ids only** — there are no names in the payload. The
`name` field is filled from a bundled mirror of Roomvana's taxonomy (see
`src/labels.rs`). An id that Roomvana adds later still works: it is listed with
a title-cased fallback name. Slugs are always the source of truth and are what
the design page accepts.

## License

MIT
