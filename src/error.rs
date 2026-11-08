use std::fmt;

/// Which kind of catalog slug a caller passed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlugKind {
    /// A space id accepted by the `room` query parameter (interior room,
    /// exterior or garden).
    Room,
    /// A style id accepted by the `style` query parameter.
    Style,
}

impl fmt::Display for SlugKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SlugKind::Room => "room",
            SlugKind::Style => "style",
        })
    }
}

/// Errors returned by this crate.
///
/// The three variants are deliberately distinct so callers can tell a
/// transport problem from a changed API payload from a bad slug of their own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The request to `https://api.roomvana.co/options` could not be completed
    /// (DNS, TLS, connection reset, timeout, non-2xx status).
    Network(String),
    /// A response body arrived but was not the expected JSON object, or it
    /// contained no usable catalog entries.
    Parse(String),
    /// The caller supplied a room or style slug that the catalog does not
    /// contain. `suggestion` holds the closest valid slug when the input looks
    /// like a typo, and `available` holds every valid slug of that kind.
    UnknownSlug {
        kind: SlugKind,
        value: String,
        suggestion: Option<String>,
        available: Vec<String>,
    },
}

impl Error {
    /// The closest valid slug offered in an [`Error::UnknownSlug`], if any.
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Error::UnknownSlug { suggestion, .. } => suggestion.as_deref(),
            _ => None,
        }
    }

    /// Every valid slug of the offending kind, in [`Error::UnknownSlug`].
    pub fn available(&self) -> &[String] {
        match self {
            Error::UnknownSlug { available, .. } => available,
            _ => &[],
        }
    }

    /// True for [`Error::UnknownSlug`], false for the transport/parse variants.
    pub fn is_unknown_slug(&self) -> bool {
        matches!(self, Error::UnknownSlug { .. })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Network(message) => {
                write!(f, "network error while contacting Roomvana: {message}")
            }
            Error::Parse(message) => write!(f, "invalid response from Roomvana: {message}"),
            Error::UnknownSlug {
                kind,
                value,
                suggestion,
                available,
            } => {
                write!(f, "unknown {kind} `{value}`")?;
                if let Some(suggestion) = suggestion {
                    write!(f, " (did you mean `{suggestion}`?)")?;
                }
                if available.is_empty() {
                    write!(f, "; the catalog has no {kind}s")
                } else {
                    write!(f, "; available {kind}s: {}", available.join(", "))
                }
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Parse(error.to_string())
    }
}
