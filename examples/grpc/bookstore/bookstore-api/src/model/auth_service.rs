use bomboni_request::{derive::Parse, query::list::ListQuery};

use crate::v1::{AuthenticateUserRequest, GetCurrentUserRequest, RegisterUserRequest};

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = RegisterUserRequest, request, write)]
pub struct ParsedRegisterUserRequest {
    pub display_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = AuthenticateUserRequest, request, write)]
pub struct ParsedAuthenticateUserRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = GetCurrentUserRequest, request, write)]
pub struct ParsedGetCurrentUserRequest {}
