use std::fmt::Debug;
use std::sync::Arc;

use http::HeaderMap;
use http::header::{AUTHORIZATION, COOKIE};
use tonic::metadata::MetadataMap;

use crate::auth::access_token::AccessTokenModel;

/// The prefix used for bearer tokens in authorization headers.
pub const BEARER_PREFIX: &str = "Bearer";

/// Trait for authenticating access tokens from various sources.
pub trait Authenticator: Debug {
    /// Authenticates a raw authorization string.
    ///
    /// # Arguments
    ///
    /// * `authorization` - The raw authorization string (typically a JWT)
    ///
    /// # Returns
    ///
    /// An `AccessTokenModel` if authentication succeeds, `None` otherwise.
    fn authenticate(&self, authorization: &str) -> Option<AccessTokenModel>;

    /// Authenticates from gRPC metadata.
    ///
    /// Extracts the bearer token from either the Authorization header or
    /// from cookies and attempts to authenticate it.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The gRPC metadata map containing authentication headers
    ///
    /// # Returns
    ///
    /// An `AccessTokenModel` if authentication succeeds, `None` otherwise.
    fn authenticate_metadata(&self, metadata: &MetadataMap) -> Option<AccessTokenModel> {
        let bearer: String = metadata
            .get(AUTHORIZATION.as_str())
            .and_then(|value| value.to_str().ok().map(ToString::to_string))
            .or_else(|| {
                metadata
                    .get_all(COOKIE.as_str())
                    .iter()
                    .find_map(|header_value| {
                        let cookie = cookie::Cookie::parse(header_value.to_str().ok()?).ok()?;
                        let (name, value) = cookie.name_value();
                        if name == AUTHORIZATION.as_str() {
                            Some(value.into())
                        } else {
                            None
                        }
                    })
            })?;
        self.authenticate_bearer(&bearer)
    }

    /// Authenticates from HTTP headers.
    ///
    /// Extracts the bearer token from either the Authorization header or
    /// from cookies and attempts to authenticate it.
    ///
    /// # Arguments
    ///
    /// * `headers` - The HTTP header map containing authentication headers
    ///
    /// # Returns
    ///
    /// An `AccessTokenModel` if authentication succeeds, `None` otherwise.
    fn authenticate_headers(&self, headers: &HeaderMap) -> Option<AccessTokenModel> {
        let bearer: String = headers
            .get(AUTHORIZATION.as_str())
            .and_then(|value| value.to_str().ok().map(ToString::to_string))
            .or_else(|| {
                headers
                    .get_all(COOKIE.as_str())
                    .iter()
                    .find_map(|header_value| {
                        let cookie = cookie::Cookie::parse(header_value.to_str().ok()?).ok()?;
                        let (name, value) = cookie.name_value();
                        if name == AUTHORIZATION.as_str() {
                            Some(value.into())
                        } else {
                            None
                        }
                    })
            })?;
        self.authenticate_bearer(&bearer)
    }

    /// Authenticates a bearer token string.
    ///
    /// Parses the bearer token format and extracts the actual token
    /// for authentication.
    ///
    /// # Arguments
    ///
    /// * `bearer` - The bearer token string in format "Bearer <token>"
    ///
    /// # Returns
    ///
    /// An `AccessTokenModel` if authentication succeeds, `None` otherwise.
    fn authenticate_bearer(&self, bearer: &str) -> Option<AccessTokenModel> {
        let parts: Vec<_> = bearer.split(' ').collect();
        if parts.len() != 2 || parts[0] != BEARER_PREFIX {
            return None;
        }
        self.authenticate(parts[1])
    }
}

/// Thread-safe shared reference to an authenticator.
pub type AuthenticatorArc = Arc<dyn Authenticator + Send + Sync>;
