use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Values {
    #[serde(flatten)]
    inner: HashMap<String, String>,
}

impl Values {
    pub fn new() -> Self {
        Values {
            inner: HashMap::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn get(&self, k: &str) -> Option<&String> {
        self.inner.get(k)
    }

    pub fn map(&self) -> &HashMap<String, String> {
        &self.inner
    }

    pub fn map_str(&self) -> HashMap<&str, &str> {
        self.inner
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect()
    }
}

impl From<HashMap<String, String>> for Values {
    fn from(h: HashMap<String, String>) -> Self {
        Self { inner: h }
    }
}

// This kind of implicit cloning of all those strings probably isn't great, but it's mostly intended for convenience
// when writing tests.
//
// If this ends up in develop, maybe we should think about Values wrapping a HashCow?
// https://github.com/purpleprotocol/hashcow
impl From<HashMap<&str, &str>> for Values {
    fn from(h: HashMap<&str, &str>) -> Self {
        Self {
            inner: h
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

impl From<&HashMap<&str, &str>> for Values {
    fn from(h: &HashMap<&str, &str>) -> Self {
        Self {
            inner: h
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

impl From<clap::Values<'_>> for Values {
    fn from(h: clap::Values) -> Self {
        Self {
            inner: h
                .into_iter()
                .map(|s| {
                    let (k, v) = parse_value(s).unwrap();
                    (k.to_string(), v.to_string())
                })
                .collect(),
        }
    }
}

fn parse_value(s: &str) -> Result<(&str, &str), String> {
    let pos = s.find(":").ok_or(format!("Invalid value `{}`", s))?;

    let mut result = s.split_at(pos);
    result.1 = &result.1[1..];

    Ok((result.0, result.1))
}
