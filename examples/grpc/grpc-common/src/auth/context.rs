use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

use bomboni_common::id::Id;
use http::header::{AUTHORIZATION, HeaderName};
use http::{HeaderMap, HeaderValue};
use tonic::metadata::{MetadataMap, MetadataValue};

use crate::auth::access_token::AccessToken;
use crate::auth::authenticator::{AuthenticatorArc, BEARER_PREFIX};

#[derive(Debug, Clone)]
pub struct ContextBuilder {
    authenticator: AuthenticatorArc,
}

#[derive(Clone)]
pub struct Context {
    pub id: Id,
    pub access_token: Option<AccessToken>,
}

impl ContextBuilder {
    pub fn new(authenticator: AuthenticatorArc) -> Self {
        ContextBuilder { authenticator }
    }

    pub fn build_from_metadata(&self, metadata: &MetadataMap) -> Context {
        let id = tracing::Span::current()
            .id()
            .map_or(Id::new(0), |id| Id::from(id.into_u64()));
        let access_token = self.authenticator.authenticate_metadata(metadata);
        Context::new(id, access_token)
    }

    pub fn build_from_headers(&self, headers: &HeaderMap) -> Context {
        let id = tracing::Span::current()
            .id()
            .map_or(Id::new(0), |id| Id::from(id.into_u64()));
        let access_token = self.authenticator.authenticate_headers(headers);
        Context::new(id, access_token)
    }

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
    pub fn new(id: Id, access_token: Option<AccessToken>) -> Self {
        Context { id, access_token }
    }

    pub fn inject(&self) -> MetadataMap {
        let mut metadata = MetadataMap::new();
        self.inject_metadata(&mut metadata);
        metadata
    }

    pub fn inject_metadata(&self, metadata: &mut MetadataMap) {
        if let Some(access_token) = self.access_token.as_ref() {
            metadata.insert(
                AUTHORIZATION.as_str(),
                MetadataValue::from_str(&format!("{} {}", BEARER_PREFIX, access_token.encoded))
                    .unwrap(),
            );
        }
    }

    pub fn authenticate_metadata(&self) -> MetadataMap {
        let mut metadata = MetadataMap::new();
        self.authenticate_with_metadata(&mut metadata);
        metadata
    }

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
