use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::google::protobuf::FieldMask;
use crate::serde::helpers as serde_helpers;

impl FieldMask {
    /// Creates a new field mask with the given paths.
    #[must_use]
    pub const fn new(paths: Vec<String>) -> Self {
        Self { paths }
    }

    /// Checks if this field mask contains the given path.
    #[must_use]
    pub fn contains(&self, path: &str) -> bool {
        self.paths.iter().any(|s| s.as_str() == path)
    }

    /// Checks if this field mask masks the given field path.
    #[must_use]
    pub fn masks(&self, field_path: &str) -> bool {
        self.paths.iter().any(|path| {
            let mut field_steps = field_path.split('.');
            for path_step in path.split('.') {
                if Some(path_step) != field_steps.next() {
                    return false;
                }
            }
            true
        })
    }
}

impl<T, P> From<T> for FieldMask
where
    T: IntoIterator<Item = P>,
    P: ToString,
{
    fn from(paths: T) -> Self {
        Self {
            paths: paths.into_iter().map(|path| path.to_string()).collect(),
        }
    }
}

impl Serialize for FieldMask {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serde_helpers::string_list::serialize(&self.paths, serializer)
    }
}

impl<'de> Deserialize<'de> for FieldMask {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let paths: Vec<String> = serde_helpers::string_list::deserialize(deserializer)?;
        Ok(Self::new(paths))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_mask() {
        let fm = FieldMask::new(vec!["f.b".into(), "f.c".into()]);
        assert!(fm.contains("f.b"));
        assert!(fm.masks("f.b.d"));
        assert!(!fm.masks("f.d"));
        assert!(!fm.masks("f.d.a"));
    }
}
