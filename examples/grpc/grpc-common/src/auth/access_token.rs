use bomboni_common::date_time::UtcDateTime;
use bomboni_request::derive::Parse;

use crate::proto::{
    AccessToken, AccessTokenAccount, AccessTokenData, AccessTokenIdentity, EmailIdentity,
    access_token_identity,
};

/// Model representing an access token.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = AccessToken, write, serde_as)]
pub struct AccessTokenModel {
    /// Whether the token is currently active.
    pub active: bool,
    /// The client ID that requested the token.
    pub client_id: String,
    /// Optional expiration time of the token.
    #[parse(timestamp)]
    pub expiration: Option<UtcDateTime>,
    /// Optional issuance time of the token.
    #[parse(timestamp)]
    pub issued_at: Option<UtcDateTime>,
    /// The issuer of the token.
    pub issuer: String,
    /// The subject (user) of the token.
    pub subject: String,
    /// Additional data associated with the token.
    pub data: Option<AccessTokenDataModel>,
    /// The encoded JWT string (not parsed from protobuf).
    #[parse(skip)]
    pub encoded: String,
}

/// Additional data contained within an access token.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = AccessTokenData, write, serde_as)]
pub struct AccessTokenDataModel {
    /// List of identities associated with the token.
    pub identities: Vec<AccessTokenIdentityModel>,
    /// List of accounts associated with the token.
    pub accounts: Vec<AccessTokenAccountModel>,
}

/// Represents different types of identities in an access token.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(
    source = AccessTokenIdentity,
    tagged_union { oneof = access_token_identity::Kind, field = kind },
    write,
    serde_as
)]
pub enum AccessTokenIdentityModel {
    /// Email-based identity.
    #[parse(source = "Email")]
    Email(EmailIdentityModel),
}

/// Represents an email-based identity.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = EmailIdentity, write, serde_as)]
pub struct EmailIdentityModel {
    /// The unique identifier for the identity.
    #[parse(source = "id")]
    pub id: String,
    /// The email address associated with the identity.
    #[parse(source = "email")]
    pub email: String,
}

/// Represents an account associated with an access token.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = AccessTokenAccount, write, serde_as)]
pub struct AccessTokenAccountModel {
    /// The unique identifier for the account.
    #[parse(source = "id")]
    pub id: String,
}
