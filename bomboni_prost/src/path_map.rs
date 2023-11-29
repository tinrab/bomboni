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
    pub fn insert<S: ToString, T: ToString>(&mut self, matcher: S, value: T) {
        self.matchers.push((matcher.to_string(), value.to_string()));
        self.sort();
    }

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
