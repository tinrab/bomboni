use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use std::{
    fmt::{self, Debug, Formatter},
    sync::Arc,
};

use crate::auth::{access_token::AccessTokenModel, authenticator::Authenticator};

/// JWT-based authenticator for validating access tokens.
pub struct JwtAuthenticator {
    decoding_key: Arc<DecodingKey>,
    validation: Arc<Validation>,
}

impl Clone for JwtAuthenticator {
    fn clone(&self) -> Self {
        Self {
            decoding_key: Arc::clone(&self.decoding_key),
            validation: Arc::clone(&self.validation),
        }
    }
}

impl Debug for JwtAuthenticator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JwtAuthenticator").finish()
    }
}

impl JwtAuthenticator {
    /// Creates a new JWT authenticator with expiration validation.
    ///
    /// # Arguments
    ///
    /// * `secret` - The secret key used for JWT validation
    ///
    /// # Returns
    ///
    /// A new `JwtAuthenticator` that validates token expiration.
    pub fn new(secret: &str) -> Self {
        Self {
            decoding_key: Arc::new(DecodingKey::from_secret(secret.as_ref())),
            validation: Arc::new({
                let mut validation = Validation::new(Algorithm::HS256);
                validation.validate_exp = true;
                validation
            }),
        }
    }

    /// Creates a new JWT authenticator without expiration validation.
    ///
    /// # Arguments
    ///
    /// * `secret` - The secret key used for JWT validation
    ///
    /// # Returns
    ///
    /// A new `JwtAuthenticator` that does not validate token expiration.
    pub fn new_no_validation(secret: &str) -> Self {
        Self {
            decoding_key: Arc::new(DecodingKey::from_secret(secret.as_ref())),
            validation: Arc::new({
                let mut validation = Validation::new(Algorithm::HS256);
                validation.validate_exp = false;
                validation
            }),
        }
    }
}

impl Authenticator for JwtAuthenticator {
    fn authenticate(&self, authorization: &str) -> Option<AccessTokenModel> {
        match jsonwebtoken::decode::<AccessTokenModel>(
            authorization,
            &self.decoding_key,
            &self.validation,
        ) {
            Ok(token) => {
                let mut access_token = token.claims;
                access_token.encoded = authorization.into();
                Some(access_token)
            }
            Err(_err) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use bomboni_common::{date_time::UtcDateTime, id::Id};
    use jsonwebtoken::{EncodingKey, Header};
    use time::{Duration, OffsetDateTime};

    use crate::auth::access_token::{
        AccessTokenAccountModel, AccessTokenDataModel, AccessTokenIdentityModel, EmailIdentityModel,
    };

    use super::*;

    #[test]
    fn test_jwt_authenticator() {
        let secret = "test_secret_key";
        let authenticator = JwtAuthenticator::new_no_validation(secret);

        let user_id = Id::new(2);
        let access_token = AccessTokenModel {
            active: true,
            client_id: "test".to_string(),
            expiration: Some((OffsetDateTime::now_utc() + Duration::hours(1)).into()),
            issued_at: Some(UtcDateTime::now()),
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

        let json = serde_json::to_string(&access_token).unwrap();
        let encoded = jsonwebtoken::encode(
            &Header::default(),
            &access_token,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap();

        let decoded = authenticator.authenticate(&encoded);
        assert!(decoded.is_some(), "Failed to authenticate token");
        let decoded = decoded.unwrap();
        assert_eq!(decoded.subject, format!("users/{}", user_id));
        assert_eq!(decoded.issuer, "bookstore");
        assert_eq!(decoded.data.as_ref().unwrap().identities.len(), 1);
        assert_eq!(decoded.data.as_ref().unwrap().accounts.len(), 1);
    }

    #[test]
    fn test_jwt_authenticator_invalid_secret() {
        let secret = "test_secret_key";
        let wrong_secret = "wrong_secret_key";
        let authenticator = JwtAuthenticator::new_no_validation(wrong_secret);

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
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap();

        let decoded = authenticator.authenticate(&encoded);
        assert!(decoded.is_none());
    }
}
