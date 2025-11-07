use serde::{Deserialize, Serialize};

use bomboni_common::id::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub active: bool,
    pub client_id: String,
    pub expiration: i64,
    pub issued_at: i64,
    pub issuer: String,
    pub subject: String,
    pub data: AccessTokenData,
    #[serde(skip)]
    pub encoded: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessTokenData {
    pub identities: Vec<AccessTokenIdentity>,
    pub accounts: Vec<AccessTokenAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AccessTokenIdentity {
    Email { id: Id, email: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenAccount {
    pub id: Id,
}
