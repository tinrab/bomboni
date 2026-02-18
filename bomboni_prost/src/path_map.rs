/// Maps fully-qualified Protobuf type names to values.
///
/// Based on [1].
///
/// [1]: https://github.com/tokio-rs/prost/blob/v0.12.3/prost-build/src/path.rs
#[derive(Clone, Debug, Default)]
pub struct PathMap {
    matchers: Vec<(String, String)>,
}

impl PathMap {
    /// Inserts a new matcher-value pair into the path map.
    ///
    /// The matcher is a protobuf type path pattern, and the value is the
    /// corresponding Rust type or mapping. The map is automatically sorted
    /// after insertion to ensure proper matching order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bomboni_prost::path_map::PathMap;
    /// let mut map = PathMap::default();
    /// map.insert(".google.protobuf.Timestamp", "chrono::DateTime<chrono::Utc>");
    /// map.insert(".myapi.User", "crate::models::User");
    /// ```
    pub fn insert<S: ToString, T: ToString>(&mut self, matcher: S, value: T) {
        self.matchers.push((matcher.to_string(), value.to_string()));
        self.sort();
    }

    /// Gets the first matching value for a given protobuf path.
    ///
    /// Searches for the most specific matcher that matches the given path.
    /// Exact matches take precedence over prefix matches.
    ///
    /// # Arguments
    ///
    /// * `path` - The protobuf type path to match against
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(matcher, value)` if a match is found, or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bomboni_prost::path_map::PathMap;
    /// let map = PathMap::from([(".a", "A"), (".a.b", "B")]);
    /// assert_eq!(map.get_first(".a").unwrap().1, "A");  // Exact match
    /// assert_eq!(map.get_first(".a.b").unwrap().1, "B");  // More specific
    /// assert_eq!(map.get_first(".a.c").unwrap().1, "A");  // Prefix match
    /// ```
    pub fn get_first(&self, path: &str) -> Option<(&str, &str)> {
        let full_path = format!(".{}", path.trim().trim_matches('.'));
        let mut best_match = None;
        for (matcher, value) in &self.matchers {
            if matcher == &full_path {
                return Some((matcher.as_str(), value.as_str()));
            }
            if full_path.starts_with(matcher) {
                best_match = Some((matcher.as_str(), value.as_str()));
            }
        }
        best_match
    }

    /// Gets the first matching value for a specific field within a type.
    ///
    /// This is a convenience method that constructs the full path for a field
    /// and delegates to `get_first`.
    ///
    /// # Arguments
    ///
    /// * `path` - The protobuf type path
    /// * `field` - The field name within the type
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(matcher, value)` if a match is found, or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bomboni_prost::path_map::PathMap;
    /// let map = PathMap::from([(".myapi.User", "crate::User")]);
    /// assert_eq!(map.get_first_field(".myapi.User", "name").unwrap().1, "crate::User");
    /// ```
    pub fn get_first_field(&self, path: &str, field: &str) -> Option<(&str, &str)> {
        self.get_first(&format!("{path}.{field}"))
    }

    fn sort(&mut self) {
        self.matchers.sort_by(|(a, _), (b, _)| a.cmp(b));
    }
}

impl<M, S, T> From<M> for PathMap
where
    M: IntoIterator<Item = (S, T)>,
    S: ToString,
    T: ToString,
{
    fn from(values: M) -> Self {
        let mut m = Self {
            matchers: values
                .into_iter()
                .map(|(s, t)| (s.to_string(), t.to_string()))
                .collect(),
        };
        m.sort();
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let m = PathMap::from([(".a", "A"), (".b", "B"), (".a.a1", "A1")]);
        assert_eq!(m.get_first(".a").unwrap().1, "A");
        assert_eq!(m.get_first(".a.a1").unwrap().1, "A1");
        assert_eq!(m.get_first(".b.B").unwrap().1, "B");
        assert_eq!(m.get_first(".a.b").unwrap().1, "A");
    }
}
