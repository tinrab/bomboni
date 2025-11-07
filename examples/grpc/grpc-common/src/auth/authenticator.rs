use std::fmt::Debug;
use std::sync::Arc;

use http::HeaderMap;
use http::header::{AUTHORIZATION, COOKIE};
use tonic::metadata::MetadataMap;

use crate::auth::access_token::AccessToken;

pub const BEARER_PREFIX: &str = "Bearer";

pub trait Authenticator: Debug {
    fn authenticate(&self, authorization: &str) -> Option<AccessToken>;

    fn authenticate_metadata(&self, metadata: &MetadataMap) -> Option<AccessToken> {
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

    fn authenticate_headers(&self, headers: &HeaderMap) -> Option<AccessToken> {
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

    fn authenticate_bearer(&self, bearer: &str) -> Option<AccessToken> {
        let parts: Vec<_> = bearer.split(' ').collect();
        if parts.len() != 2 || parts[0] != BEARER_PREFIX {
            return None;
        }
        self.authenticate(parts[1])
    }
}

pub type AuthenticatorArc = Arc<dyn Authenticator + Send + Sync>;
