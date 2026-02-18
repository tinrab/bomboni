use jsonwebtoken::{Algorithm, DecodingKey, Validation};

use crate::auth::{access_token::AccessTokenModel, authenticator::Authenticator};

/// In-memory authenticator for testing purposes.
///
/// This authenticator uses a fixed secret key and does not validate
/// token expiration, making it suitable for development and testing.
#[derive(Debug, Default)]
pub struct MemoryAuthenticator {}

impl MemoryAuthenticator {
    /// Creates a new memory authenticator.
    ///
    /// # Returns
    ///
    /// A new `MemoryAuthenticator` instance.
    pub const fn new() -> Self {
        Self {}
    }
}

impl Authenticator for MemoryAuthenticator {
    fn authenticate(&self, authorization: &str) -> Option<AccessTokenModel> {
        let token = jsonwebtoken::decode::<AccessTokenModel>(
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

    use crate::auth::access_token::{
        AccessTokenAccountModel, AccessTokenDataModel, AccessTokenIdentityModel, EmailIdentityModel,
    };

    use super::*;

    #[test]
    fn generate_test_access_token() {
        let user_id = Id::new(2);
        let access_token = AccessTokenModel {
            active: true,
            client_id: "test".to_string(),
            expiration: None,
            issued_at: None,
            issuer: "bookstore".to_string(),
            subject: format!("users/{}", user_id),
            data: Some(AccessTokenDataModel {
                identities: vec![AccessTokenIdentityModel::Email(EmailIdentityModel {
                    id: user_id.to_string(),
                    email: format!("tester+{}@bookstore.com", user_id),
                })],
                accounts: vec![AccessTokenAccountModel {
                    id: user_id.to_string(),
                }],
            }),
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
