use std::collections::hash_map;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Values<'s> {
    #[serde(flatten, borrow)]
    inner: HashMap<&'s str, &'s str>,
}

impl Values<'_> {
    pub fn iter(&self) -> hash_map::Iter<&str, &str> {
        self.inner.iter()
    }

    pub fn get(&self, k: &str) -> Option<&&str> {
        self.inner.get(k)
    }

    pub fn map(&self) -> &HashMap<&str, &str> {
        &self.inner
    }
}

impl<'s> From<HashMap<&'s str, &'s str>> for Values<'s> {
    fn from(h: HashMap<&'s str, &'s str>) -> Self {
        Self {
            inner: h,
        }
    }
}

// Because a Rust codebase without an unsafe block is kind of dull. Right?
// ...right?
impl<'s> AsRef<Values<'s>> for HashMap<&'s str, &'s str> {
    fn as_ref(&self) -> &Values<'s> {
        unsafe {
            &*(self as *const Self as *const Values<'s>)
        }
    }
}

impl<'s> AsRef<HashMap<&'s str, &'s str>> for Values<'s> {
    fn as_ref(&self) -> &HashMap<&'s str, &'s str> {
        unsafe {
            &*(self as *const Self as *const HashMap<&'s str, &'s str>)
        }
    }
}
