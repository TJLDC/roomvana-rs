//! Display names for catalog slugs.
//!
//! `GET /options` carries **ids only** — the payload has no human-readable
//! names. This table is a hand-maintained mirror of the backend taxonomy
//! (`roomvana/backend/app/options.py`), used purely to enrich ids with a label.
//! An id that is live but absent here still works; it falls back to a
//! title-cased form of the slug. Refresh this table when the backend adds
//! spaces or styles.

pub(crate) fn space_name(slug: &str) -> Option<&'static str> {
    Some(match slug {
        // interior rooms
        "kitchen" => "kitchen",
        "living-room" => "living room",
        "bedroom" => "bedroom",
        "bathroom" => "bathroom",
        "dining-room" => "dining room",
        "home-office" => "home office",
        "kids-room" => "kids room",
        "nursery" => "nursery",
        "basement" => "basement",
        "attic" => "attic bedroom",
        "entryway" => "entryway",
        "laundry-room" => "laundry room",
        "closet" => "walk-in closet",
        "home-theater" => "home theater room",
        "home-gym" => "home gym",
        "wine-cellar" => "wine cellar",
        "basement-bar" => "basement bar",
        "staircase" => "staircase and stairwell",
        // exterior
        "exterior" => "house exterior",
        // gardens and outdoor areas
        "garden" => "garden",
        "backyard" => "backyard",
        "landscape" => "yard and surrounding landscape",
        "patio" => "patio and outdoor seating area",
        "deck" => "deck",
        "porch" => "porch",
        "balcony" => "balcony",
        _ => return None,
    })
}

pub(crate) fn style_name(slug: &str) -> Option<&'static str> {
    Some(match slug {
        "modern" => "Modern",
        "minimalist" => "Minimalist",
        "scandinavian" => "Scandinavian",
        "japandi" => "Japandi",
        "mid-century-modern" => "Mid-Century Modern",
        "industrial" => "Industrial",
        "bohemian" => "Bohemian",
        "modern-farmhouse" => "Modern Farmhouse",
        "coastal" => "Coastal",
        "traditional" => "Traditional",
        "contemporary" => "Contemporary",
        "rustic" => "Rustic",
        "french-country" => "French Country",
        "mediterranean" => "Mediterranean",
        "art-deco" => "Art Deco",
        "modern-luxury" => "Modern Luxury",
        "vintage" => "Vintage",
        "dark-academia" => "Dark Academia",
        "tropical" => "Tropical",
        "cottagecore" => "Cottagecore",
        "transitional" => "Transitional",
        "spanish-colonial-revival" => "Spanish Colonial Revival",
        "2026-trends" => "2026 Trends",
        _ => return None,
    })
}

/// Title-case a slug for ids the mirror has not caught up with:
/// `mid-century-modern` -> `Mid Century Modern`.
pub(crate) fn title_case(slug: &str) -> String {
    let mut out = String::with_capacity(slug.len());
    let mut start_of_word = true;
    for ch in slug.chars() {
        if ch == '-' || ch == '_' {
            out.push(' ');
            start_of_word = true;
        } else if start_of_word {
            out.extend(ch.to_uppercase());
            start_of_word = false;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_case_handles_hyphens_and_digits() {
        assert_eq!(title_case("mid-century-modern"), "Mid Century Modern");
        assert_eq!(title_case("2026-trends"), "2026 Trends");
        assert_eq!(title_case("kitchen"), "Kitchen");
    }
}
