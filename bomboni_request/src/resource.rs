//! # Tools for working with API resources.

pub use bomboni_derive::parse_resource_name;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_names() {
        let f = parse_resource_name!([
            "users" => u32,
            "projects" => u64,
            "revisions" => Option<String>,
        ]);

        let (user_id, project_id, revision_id) = f("users/3/projects/5/revisions/1337").unwrap();
        assert_eq!(user_id, 3);
        assert_eq!(project_id, 5);
        assert_eq!(revision_id, Some("1337".to_string()));

        let (user_id, project_id, revision_id) = f("users/3/projects/5").unwrap();
        assert_eq!(user_id, 3);
        assert_eq!(project_id, 5);
        assert!(revision_id.is_none());

        assert!(parse_resource_name!([
            "a" => u32,
            "b" => u32,
        ])("a/1/b/1/c/1")
        .is_none());
    }
}
