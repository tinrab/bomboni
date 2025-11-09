use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Formatter},
    str::FromStr,
};

use bomboni_common::id::Id;
use http::{
    HeaderMap, HeaderValue,
    header::{AUTHORIZATION, HeaderName},
};
use tonic::metadata::{MetadataMap, MetadataValue};

use crate::auth::{
    access_token::AccessTokenModel,
    authenticator::{AuthenticatorArc, BEARER_PREFIX},
};

/// Builder for creating authentication contexts.
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    authenticator: AuthenticatorArc,
}

/// Authentication context containing request ID and access token.
#[derive(Clone)]
pub struct Context {
    /// Unique identifier for the request/context.
    pub id: Id,
    /// The authenticated access token, if any.
    pub access_token: Option<AccessTokenModel>,
}

impl ContextBuilder {
    /// Creates a new context builder.
    ///
    /// # Arguments
    ///
    /// * `authenticator` - The authenticator to use for token validation
    pub fn new(authenticator: AuthenticatorArc) -> Self {
        Self { authenticator }
    }

    /// Builds a context from gRPC metadata.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The gRPC metadata containing authentication information
    ///
    /// # Returns
    ///
    /// A new `Context` with the extracted authentication information.
    pub fn build_from_metadata(&self, metadata: &MetadataMap) -> Context {
        let id = tracing::Span::current()
            .id()
            .map_or(Id::new(0), |id| Id::from(id.into_u64()));
        let access_token = self.authenticator.authenticate_metadata(metadata);
        Context::new(id, access_token)
    }

    /// Builds a context from HTTP headers.
    ///
    /// # Arguments
    ///
    /// * `headers` - The HTTP headers containing authentication information
    ///
    /// # Returns
    ///
    /// A new `Context` with the extracted authentication information.
    pub fn build_from_headers(&self, headers: &HeaderMap) -> Context {
        let id = tracing::Span::current()
            .id()
            .map_or(Id::new(0), |id| Id::from(id.into_u64()));
        let access_token = self.authenticator.authenticate_headers(headers);
        Context::new(id, access_token)
    }

    /// Builds a context from a header map.
    ///
    /// # Arguments
    ///
    /// * `headers` - A map of header names to sets of header values
    ///
    /// # Returns
    ///
    /// A new `Context` with the extracted authentication information.
    ///
    /// # Panics
    ///
    /// Will panic if any header name is invalid.
    pub fn build_from_header_map(&self, headers: &HashMap<String, HashSet<String>>) -> Context {
        let header_map = headers
            .iter()
            .map(|(k, v)| {
                (
                    HeaderName::from_str(k).expect("invalid header name"),
                    HeaderValue::from_str(&v.iter().cloned().collect::<Vec<_>>().join(","))
                        .unwrap(),
                )
            })
            .collect();
        self.build_from_headers(&header_map)
    }
}

impl Context {
    /// Creates a new context.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for the context
    /// * `access_token` - Optional access token for authentication
    pub const fn new(id: Id, access_token: Option<AccessTokenModel>) -> Self {
        Self { id, access_token }
    }

    /// Creates metadata map with authentication injected.
    ///
    /// # Returns
    ///
    /// A new `MetadataMap` containing the authentication information.
    pub fn inject(&self) -> MetadataMap {
        let mut metadata = MetadataMap::new();
        self.inject_metadata(&mut metadata);
        metadata
    }

    /// Injects authentication metadata into the provided metadata map.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata map to inject authentication into
    ///
    /// # Panics
    ///
    /// Will panic if the access token contains invalid characters for metadata values.
    pub fn inject_metadata(&self, metadata: &mut MetadataMap) {
        if let Some(access_token) = self.access_token.as_ref() {
            metadata.insert(
                AUTHORIZATION.as_str(),
                MetadataValue::from_str(&format!("{} {}", BEARER_PREFIX, access_token.encoded))
                    .unwrap(),
            );
        }
    }

    /// Creates metadata map with authentication for authentication purposes.
    ///
    /// # Returns
    ///
    /// A new `MetadataMap` containing the authentication information.
    pub fn authenticate_metadata(&self) -> MetadataMap {
        let mut metadata = MetadataMap::new();
        self.authenticate_with_metadata(&mut metadata);
        metadata
    }

    /// Authenticates the provided metadata map.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata map to authenticate
    ///
    /// # Panics
    ///
    /// Will panic if the access token contains invalid characters for metadata values.
    pub fn authenticate_with_metadata(&self, metadata: &mut MetadataMap) {
        if let Some(access_token) = self.access_token.as_ref() {
            metadata.insert(
                AUTHORIZATION.as_str(),
                MetadataValue::from_str(&format!("{} {}", BEARER_PREFIX, access_token.encoded))
                    .unwrap(),
            );
        }
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Context");
        d.finish()
    }
}
