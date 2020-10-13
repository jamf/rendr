use std::fmt;
use std::ops::Deref;

use serde::de::{self, Deserialize, Deserializer, Visitor};

/// A glob pattern for paths, meant to be directly deserialized into.
///
/// This is effectively a wrapper around patterns in the `glob` crate and follows
/// [the same rules](glob::Pattern).
pub struct Pattern(glob::Pattern);

// Implementing Deref makes our Pattern wrapper almost a drop-in replacement
// for glob::Pattern.
impl Deref for Pattern {
    type Target = glob::Pattern;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Enable deserialization from a string.
struct PatternVisitor;

impl<'de> Visitor<'de> for PatternVisitor {
    type Value = Pattern;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing a path pattern")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Pattern(match glob::Pattern::new(value) {
            Ok(pattern) => pattern,
            Err(e) => return Err(E::custom(format!("invalid path pattern {}: {}", value, e))),
        }))
    }
}

impl<'de> Deserialize<'de> for Pattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PatternVisitor)
    }
}
