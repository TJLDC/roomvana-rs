//! A small, synchronous Rust client for the public Roomvana design catalog.
//!
//! Roomvana (<https://roomvana.ai>) is an AI interior design tool. Its public,
//! no-auth endpoint `GET https://api.roomvana.co/options` lists the space types
//! and design styles the product can render. This crate reads that catalog and
//! builds design links of the form
//! `https://roomvana.ai/design?room=<room>&style=<style>`.
//!
//! It only reads the catalog: it never generates an image, never spends
//! credits, and needs no API key.
//!
//! ```
//! # fn main() -> Result<(), roomvana::Error> {
//! use roomvana::Catalog;
//!
//! // `from_json` parses a payload already in hand (or a fixture in tests).
//! let catalog = Catalog::from_json(
//!     r#"{"rooms":["kitchen"],"exteriors":["exterior"],
//!         "gardens":["garden"],"styles":["japandi"]}"#,
//! )?;
//!
//! assert_eq!(catalog.rooms()[0].name, "kitchen");
//! assert_eq!(catalog.styles()[0].name, "Japandi");
//! assert_eq!(
//!     catalog.design_link("kitchen", "japandi")?,
//!     "https://roomvana.ai/design?room=kitchen&style=japandi",
//! );
//! # Ok(())
//! # }
//! ```
//!
//! For the live catalog use [`Catalog::fetch`]. It performs one blocking HTTPS
//! request; there is no async runtime and no hidden cache.

mod error;
mod labels;

pub use error::{Error, SlugKind};

use std::fmt::Write as _;
use std::time::Duration;

use serde::Deserialize;

/// Base URL of Roomvana's API. The `api.roomvana.co` host is permanent: the
/// shipped iOS build has it compiled in.
pub const API_BASE: &str = "https://api.roomvana.co";

/// Base URL of the Roomvana web app.
pub const SITE_BASE: &str = "https://roomvana.ai";

/// Request timeout used by [`Catalog::fetch`].
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// A space that the design studio accepts as `?room=`.
///
/// This covers interior rooms as well as the exterior and garden ids; they are
/// all values of the same query parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    /// The id to put in the `room` query parameter, e.g. `living-room`.
    pub slug: String,
    /// A human-readable name, e.g. `living room`. `GET /options` carries ids
    /// only, so this comes from a bundled mirror of the backend taxonomy
    /// (title-cased slug as the fallback for ids the mirror does not know).
    pub name: String,
}

/// A design style that the studio accepts as `?style=`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    /// The id to put in the `style` query parameter, e.g. `mid-century-modern`.
    pub slug: String,
    /// A human-readable name, e.g. `Mid-Century Modern` (same bundled-mirror
    /// caveat as [`Room::name`]).
    pub name: String,
}

/// The room/style catalog served by `GET https://api.roomvana.co/options`.
///
/// The payload also carries surfaces, finishes, shades and capability flags;
/// this crate intentionally exposes only the design-page taxonomy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    rooms: Vec<String>,
    exteriors: Vec<String>,
    gardens: Vec<String>,
    styles: Vec<String>,
}

#[derive(Deserialize)]
struct RawOptions {
    #[serde(default)]
    rooms: Vec<String>,
    #[serde(default)]
    exteriors: Vec<String>,
    #[serde(default)]
    gardens: Vec<String>,
    #[serde(default)]
    styles: Vec<String>,
}

impl Catalog {
    /// Fetch the live catalog from `https://api.roomvana.co/options`.
    ///
    /// Blocking, with a 10 second timeout. No API key is required and nothing
    /// is billed. The call is not cached — callers that make several lookups
    /// should fetch once and reuse the [`Catalog`].
    pub fn fetch() -> Result<Catalog, Error> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(DEFAULT_TIMEOUT))
            .user_agent(concat!("roomvana-rs/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        let mut response = agent
            .get(format!("{API_BASE}/options"))
            .call()
            .map_err(|error| Error::Network(error.to_string()))?;

        let body = response
            .body_mut()
            .read_to_string()
            .map_err(|error| Error::Network(format!("could not read the response body: {error}")))?;

        Catalog::from_json(&body)
    }

    /// Parse a catalog payload that is already in hand, e.g. a saved response
    /// or a test fixture. Unknown keys are ignored.
    pub fn from_json(json: &str) -> Result<Catalog, Error> {
        let raw: RawOptions = serde_json::from_str(json)?;
        let catalog = Catalog {
            rooms: clean(raw.rooms),
            exteriors: clean(raw.exteriors),
            gardens: clean(raw.gardens),
            styles: clean(raw.styles),
        };

        if catalog.styles.is_empty() {
            return Err(Error::Parse(
                "payload contains no styles; expected a non-empty `styles` array".into(),
            ));
        }
        if catalog.rooms.is_empty() && catalog.exteriors.is_empty() && catalog.gardens.is_empty() {
            return Err(Error::Parse(
                "payload contains no spaces; expected non-empty `rooms`, `exteriors` or `gardens`"
                    .into(),
            ));
        }
        Ok(catalog)
    }

    /// Interior room ids, exactly as served under `rooms`.
    pub fn room_slugs(&self) -> &[String] {
        &self.rooms
    }

    /// Exterior ids, exactly as served under `exteriors`.
    pub fn exterior_slugs(&self) -> &[String] {
        &self.exteriors
    }

    /// Garden and outdoor ids, exactly as served under `gardens`.
    pub fn garden_slugs(&self) -> &[String] {
        &self.gardens
    }

    /// Style ids, exactly as served under `styles`.
    pub fn style_slugs(&self) -> &[String] {
        &self.styles
    }

    /// Interior rooms with display names.
    pub fn rooms(&self) -> Vec<Room> {
        self.rooms.iter().map(|slug| room(slug)).collect()
    }

    /// Exterior spaces with display names.
    pub fn exteriors(&self) -> Vec<Room> {
        self.exteriors.iter().map(|slug| room(slug)).collect()
    }

    /// Garden and outdoor spaces with display names.
    pub fn gardens(&self) -> Vec<Room> {
        self.gardens.iter().map(|slug| room(slug)).collect()
    }

    /// Every space the design page accepts as `?room=` (interior, exterior and
    /// garden ids, in that order).
    pub fn spaces(&self) -> Vec<Room> {
        self.all_space_slugs()
            .iter()
            .map(|slug| room(slug))
            .collect()
    }

    /// Styles with display names.
    pub fn styles(&self) -> Vec<Style> {
        self.styles
            .iter()
            .map(|slug| Style {
                slug: slug.clone(),
                name: labels::style_name(slug)
                    .map(str::to_owned)
                    .unwrap_or_else(|| labels::title_case(slug)),
            })
            .collect()
    }

    /// Whether `slug` is a space id the design page accepts.
    pub fn contains_space(&self, slug: &str) -> bool {
        self.rooms.iter().any(|id| id == slug)
            || self.exteriors.iter().any(|id| id == slug)
            || self.gardens.iter().any(|id| id == slug)
    }

    /// Whether `slug` is a style id the design page accepts.
    pub fn contains_style(&self, slug: &str) -> bool {
        self.styles.iter().any(|id| id == slug)
    }

    /// Build `https://roomvana.ai/design?room=<room>&style=<style>`.
    ///
    /// Both slugs are validated against this catalog. An unknown slug returns
    /// [`Error::UnknownSlug`] carrying the nearest valid value (when the input
    /// looks like a typo) and the full list of valid values.
    pub fn design_link(&self, room: &str, style: &str) -> Result<String, Error> {
        if !self.contains_space(room) {
            let available = self.all_space_slugs();
            return Err(Error::UnknownSlug {
                kind: SlugKind::Room,
                value: room.to_owned(),
                suggestion: nearest(room, &available),
                available,
            });
        }
        if !self.contains_style(style) {
            return Err(Error::UnknownSlug {
                kind: SlugKind::Style,
                value: style.to_owned(),
                suggestion: nearest(style, &self.styles),
                available: self.styles.clone(),
            });
        }

        Ok(format!(
            "{SITE_BASE}/design?room={}&style={}",
            encode_query_value(room),
            encode_query_value(style)
        ))
    }

    fn all_space_slugs(&self) -> Vec<String> {
        let mut all = Vec::with_capacity(self.rooms.len() + self.exteriors.len() + self.gardens.len());
        all.extend_from_slice(&self.rooms);
        all.extend_from_slice(&self.exteriors);
        all.extend_from_slice(&self.gardens);
        all
    }
}

fn clean(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect()
}

fn room(slug: &str) -> Room {
    Room {
        slug: slug.to_owned(),
        name: labels::space_name(slug)
            .map(str::to_owned)
            .unwrap_or_else(|| labels::title_case(slug)),
    }
}

/// Percent-encode a query value. Catalog slugs are already URL-safe, but the
/// encoder keeps the output correct if the server ever serves something else.
fn encode_query_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

/// Closest candidate by edit distance, with a preference for prefix matches.
/// Returns `None` when nothing is close enough to be a plausible typo.
fn nearest(value: &str, candidates: &[String]) -> Option<String> {
    let needle = value.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return None;
    }

    let mut best: Option<(usize, &String)> = None;
    for candidate in candidates {
        let lowered = candidate.to_ascii_lowercase();
        let distance = if lowered == needle {
            0
        } else if lowered.starts_with(&needle) || needle.starts_with(&lowered) {
            1
        } else {
            edit_distance(&needle, &lowered)
        };
        if best.is_none_or(|(current, _)| distance < current) {
            best = Some((distance, candidate));
        }
    }

    best.filter(|(distance, _)| *distance <= 3)
        .map(|(_, candidate)| candidate.clone())
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];

    for (i, ca) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != cb);
            current[j + 1] = substitution.min(previous[j + 1] + 1).min(current[j] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_distance_is_symmetric_enough_for_typos() {
        assert_eq!(edit_distance("kitchn", "kitchen"), 1);
        assert_eq!(edit_distance("japandi", "japandi"), 0);
        assert_eq!(edit_distance("", "modern"), 6);
    }

    #[test]
    fn query_values_are_percent_encoded() {
        assert_eq!(encode_query_value("living-room"), "living-room");
        assert_eq!(encode_query_value("a b&c"), "a%20b%26c");
    }
}
