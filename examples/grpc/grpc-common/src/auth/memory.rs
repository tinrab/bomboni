use jsonwebtoken::{Algorithm, DecodingKey, Validation};

use crate::auth::access_token::AccessToken;
use crate::auth::authenticator::Authenticator;

#[derive(Debug, Default)]
pub struct MemoryAuthenticator {}

impl MemoryAuthenticator {
    pub fn new() -> Self {
        MemoryAuthenticator {}
    }
}

impl Authenticator for MemoryAuthenticator {
    fn authenticate(&self, authorization: &str) -> Option<AccessToken> {
        let token = jsonwebtoken::decode::<AccessToken>(
            authorization,
            &DecodingKey::from_secret("testsecret".as_ref()),
            &{
                let mut validation = Validation::new(Algorithm::HS256);
                validation.validate_exp = false;
                validation
            },
        )
        .ok()?;
        let mut access_token = token.claims;
        access_token.encoded = authorization.into();
        Some(access_token)
    }
}

#[cfg(test)]
mod tests {
    use bomboni_common::id::Id;
    use jsonwebtoken::{EncodingKey, Header};

    use crate::auth::access_token::{AccessTokenAccount, AccessTokenData, AccessTokenIdentity};

    use super::*;

    #[test]
    fn generate_test_access_token() {
        let user_id = Id::new(2);
        let access_token = AccessToken {
            active: true,
            client_id: "test".to_string(),
            expiration: 0,
            issued_at: 0,
            issuer: "bookstore".to_string(),
            subject: format!("users/{}", user_id),
            data: AccessTokenData {
                identities: vec![AccessTokenIdentity::Email {
                    id: user_id,
                    email: format!("tester+{}@bookstore.com", user_id),
                }],
                accounts: vec![AccessTokenAccount { id: user_id }],
            },
            encoded: String::new(),
        };
        let encoded = jsonwebtoken::encode(
            &Header::default(),
            &access_token,
            &EncodingKey::from_secret("bookstore_secret".as_ref()),
        )
        .unwrap();
        println!("Example access token: {}", encoded);
    }
}
