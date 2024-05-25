use crate::{
    filter::Filter,
    ordering::Ordering,
    query::{
        error::{QueryError, QueryResult},
        page_token::{utility::get_page_filter, FilterPageToken, PageTokenBuilder},
    },
};
use base64ct::{Base64, Base64Url, Encoding};
use std::fmt::{self, Debug, Formatter};

/// Page token builder for Base64-encoded tokens.
/// Used only in insecure environments.
#[derive(Clone)]
pub struct Base64PageTokenBuilder {
    url_safe: bool,
}

impl Base64PageTokenBuilder {
    pub fn new(url_safe: bool) -> Self {
        Self { url_safe }
    }
}

impl PageTokenBuilder for Base64PageTokenBuilder {
    type PageToken = FilterPageToken;

    fn parse(
        &self,
        _filter: &Filter,
        _ordering: &Ordering,
        _salt: &[u8],
        page_token: &str,
    ) -> QueryResult<Self::PageToken> {
        let decoded = if self.url_safe {
            Base64Url::decode_vec(page_token).map_err(|_| QueryError::InvalidPageToken)?
        } else {
            Base64::decode_vec(page_token).map_err(|_| QueryError::InvalidPageToken)?
        };
        let page_filter =
            Filter::parse(&String::from_utf8(decoded).map_err(|_| QueryError::InvalidPageToken)?)?;
        Ok(Self::PageToken {
            filter: page_filter,
        })
    }

    fn build_next<T: crate::schema::SchemaMapped>(
        &self,
        _filter: &Filter,
        ordering: &Ordering,
        _salt: &[u8],
        next_item: &T,
    ) -> QueryResult<String> {
        let page_filter = get_page_filter(ordering, next_item);
        if page_filter.is_empty() {
            return Err(QueryError::PageTokenFailure);
        }
        if self.url_safe {
            Ok(Base64Url::encode_string(page_filter.to_string().as_bytes()))
        } else {
            Ok(Base64::encode_string(page_filter.to_string().as_bytes()))
        }
    }
}

impl Debug for Base64PageTokenBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Base64PageTokenBuilder").finish()
    }
}

#[cfg(feature = "testing")]
#[cfg(test)]
mod tests {
    use crate::testing::schema::UserItem;

    use super::*;

    #[test]
    fn it_works() {
        let b = Base64PageTokenBuilder::new(true);
        let filter = Filter::parse(r#"displayName = "John""#).unwrap();
        let ordering = Ordering::parse("id desc, age desc").unwrap();
        let page_token = b
            .build_next(
                &filter,
                &ordering,
                &[],
                &UserItem {
                    id: "1337".into(),
                    display_name: "John".into(),
                    age: 14000,
                },
            )
            .unwrap();
        assert_eq!(page_token, "aWQgPD0gIjEzMzciIEFORCBhZ2UgPD0gMTQwMDA=");
        let parsed = b.parse(&filter, &ordering, &[], &page_token).unwrap();
        assert_eq!(
            parsed.filter.to_string(),
            r#"id <= "1337" AND age <= 14000"#
        );
    }
}
