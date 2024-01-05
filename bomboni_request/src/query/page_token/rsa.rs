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
};
use base64ct::{Base64, Base64Url, Encoding};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::fmt::{self, Debug, Formatter};

const PARAMS_KEY_LENGTH: usize = 32;

/// Page token builder for RSA-encrypted tokens.
#[derive(Clone)]
pub struct RsaPageTokenBuilder {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
    url_safe: bool,
}

impl RsaPageTokenBuilder {
    pub fn new(private_key: RsaPrivateKey, public_key: RsaPublicKey, url_safe: bool) -> Self {
        Self {
            private_key,
            public_key,
            url_safe,
        }
    }
}

impl PageTokenBuilder for RsaPageTokenBuilder {
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

        let mut plaintext = self
            .private_key
            .decrypt(Pkcs1v15Encrypt, &decoded)
            .map_err(|_| QueryError::InvalidPageToken)?;

        // Verify key
        let page_filter_text = plaintext.split_off(PARAMS_KEY_LENGTH);
        let params_key = make_page_key::<PARAMS_KEY_LENGTH>(filter, ordering, salt);
        if params_key != plaintext.as_slice() {
            return Err(QueryError::InvalidPageToken);
        }

        let page_filter = Filter::parse(
            &String::from_utf8(page_filter_text).map_err(|_| QueryError::InvalidPageToken)?,
        )
        .map_err(|_| QueryError::InvalidPageToken)?;

        Ok(Self::PageToken {
            filter: page_filter,
        })
    }

    fn build_next<T: crate::schema::SchemaMapped>(
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

        // Include both filter and ordering into encryption.
        let mut plaintext = make_page_key::<PARAMS_KEY_LENGTH>(filter, ordering, salt).to_vec();
        plaintext.extend(page_filter.to_string().as_bytes());

        let mut rng = rand::thread_rng();
        let encrypted = self
            .public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, &plaintext)
            .unwrap();

        if self.url_safe {
            Ok(Base64Url::encode_string(&encrypted))
        } else {
            Ok(Base64::encode_string(&encrypted))
        }
    }
}

impl Debug for RsaPageTokenBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RsaPageTokenBuilder").finish()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use crate::testing::schema::UserItem;

    use super::*;

    #[test]
    fn it_works() {
        let b = get_builder();
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
        assert!(page_token.trim().len() > 16);
        assert!(!page_token.contains(r#"id <= "1337""#));
        let parsed = b.parse(&filter, &ordering, &[], &page_token).unwrap();
        assert_eq!(
            parsed.filter.to_string(),
            r#"id <= "1337" AND age <= 14000"#
        );
    }

    #[test]
    fn errors() {
        let b = get_builder();

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

    fn get_builder() -> &'static RsaPageTokenBuilder {
        static SINGLETON: OnceLock<RsaPageTokenBuilder> = OnceLock::new();
        SINGLETON.get_or_init(|| {
            let mut rng = rand::thread_rng();
            let private_key = RsaPrivateKey::new(&mut rng, 720).unwrap();
            let public_key = RsaPublicKey::from(&private_key);
            RsaPageTokenBuilder::new(private_key, public_key, true)
        })
    }
}
