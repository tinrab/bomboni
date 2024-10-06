use crate::{
    filter::Filter,
    ordering::Ordering,
    query::{
        error::{QueryError, QueryResult},
        page_token::{
            utility::{get_page_filter, make_page_key},
            FilterPageToken, PageTokenBuilder,
        },
    },
    schema::SchemaMapped,
};
use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use base64ct::{Base64, Base64Url, Encoding};
use std::fmt::{self, Debug, Formatter};

const NONCE_LENGTH: usize = 12;

/// AES-256-GCM page token builder.
/// The page token is encrypted using the query parameters as the key.
/// This is useful for ensuring that the page token was generated for the same paging rules.
#[derive(Clone)]
pub struct Aes256PageTokenBuilder {
    url_safe: bool,
}

impl Aes256PageTokenBuilder {
    pub fn new(url_safe: bool) -> Self {
        Self { url_safe }
    }
}

impl PageTokenBuilder for Aes256PageTokenBuilder {
    type PageToken = FilterPageToken;

    fn parse(
        &self,
        filter: &Filter,
        ordering: &Ordering,
        salt: &[u8],
        page_token: &str,
    ) -> QueryResult<Self::PageToken> {
        let decoded = if self.url_safe {
            Base64Url::decode_vec(page_token).map_err(|_| QueryError::InvalidPageToken)?
        } else {
            Base64::decode_vec(page_token).map_err(|_| QueryError::InvalidPageToken)?
        };

        let key = make_page_key::<32>(filter, ordering, salt);
        let key: &Key<Aes256Gcm> = (&key).into();

        let cipher = Aes256Gcm::new(key);
        if decoded.len() <= NONCE_LENGTH {
            return Err(QueryError::InvalidPageToken);
        }
        let (nonce_buf, encrypted) = decoded.split_at(NONCE_LENGTH);

        let plaintext = cipher
            .decrypt(nonce_buf.into(), encrypted)
            .map_err(|_| QueryError::InvalidPageToken)?;

        let page_filter =
            Filter::parse(&String::from_utf8(plaintext).map_err(|_| QueryError::InvalidPageToken)?)
                .map_err(|_| QueryError::InvalidPageToken)?;

        Ok(Self::PageToken {
            filter: page_filter,
        })
    }

    fn build_next<T: SchemaMapped>(
        &self,
        filter: &Filter,
        ordering: &Ordering,
        salt: &[u8],
        next_item: &T,
    ) -> QueryResult<String> {
        let page_filter = get_page_filter(ordering, next_item);
        if page_filter.is_empty() {
            return Err(QueryError::PageTokenFailure);
        }
        let plaintext = page_filter.to_string();

        let key = make_page_key::<32>(filter, ordering, salt);
        let key: &Key<Aes256Gcm> = (&key).into();

        let cipher = Aes256Gcm::new(key);
        // 96-bits; unique per message
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let mut encrypted = cipher.encrypt(&nonce, plaintext.as_bytes()).unwrap();
        // Prepend nonce to encrypted buffer
        encrypted.splice(0..0, nonce);

        if self.url_safe {
            Ok(Base64Url::encode_string(&encrypted))
        } else {
            Ok(Base64::encode_string(&encrypted))
        }
    }
}

impl Debug for Aes256PageTokenBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Aes256PageTokenBuilder").finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::schema::UserItem;

    use super::*;

    #[test]
    fn it_works() {
        let b = Aes256PageTokenBuilder::new(true);
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
        assert!(page_token.trim().len() > NONCE_LENGTH);
        assert!(!page_token.contains(r#"id <= "1337""#));
        let parsed = b.parse(&filter, &ordering, &[], &page_token).unwrap();
        assert_eq!(
            parsed.filter.to_string(),
            r#"id <= "1337" AND age <= 14000"#
        );
    }

    #[test]
    fn errors() {
        let b = Aes256PageTokenBuilder::new(true);

        // Generate key for different parameters
        let filter = Filter::parse("id=1").unwrap();
        let ordering = Ordering::parse("age desc").unwrap();
        let salt = "salt".as_bytes();
        let page_token = b
            .build_next(
                &filter,
                &ordering,
                salt,
                &UserItem {
                    id: "1337".into(),
                    display_name: "John".into(),
                    age: 14000,
                },
            )
            .unwrap();
        let parsed = b.parse(&filter, &ordering, salt, &page_token).unwrap();
        assert_eq!(parsed.filter.to_string(), "age <= 14000");
        assert_eq!(
            b.parse(
                &Filter::parse("id=2").unwrap(),
                &Ordering::parse("age desc").unwrap(),
                salt,
                &page_token
            )
            .unwrap_err(),
            QueryError::InvalidPageToken
        );
        assert_eq!(
            b.parse(
                &Filter::parse("id=1").unwrap(),
                &Ordering::parse("age asc").unwrap(),
                salt,
                &page_token
            )
            .unwrap_err(),
            QueryError::InvalidPageToken
        );
        assert_eq!(
            b.parse(&filter, &ordering, &[], &page_token).unwrap_err(),
            QueryError::InvalidPageToken
        );
    }
}
