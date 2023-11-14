///Implement [`prost::Name`] for `Command`.
impl ::prost::Name for Command {
    const NAME: &'static str = "Command";
    const PACKAGE: &'static str = "tools.command";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl Command {
    pub const TYPE_URL: &'static str = "/Command";
}
impl Command {
    pub const USER_FIELD_NAME: &'static str = "user";
    pub const STATUS_FIELD_NAME: &'static str = "status";
    pub const PRINT_FIELD_NAME: &'static str = "print";
    pub const APPLY_PERMS_FIELD_NAME: &'static str = "apply_perms";
}
impl Command {
    pub const KIND_ONEOF_NAME: &'static str = "kind";
}
impl command::Kind {
    pub const STATUS_VARIANT_NAME: &'static str = "status";
    pub const PRINT_VARIANT_NAME: &'static str = "print";
    pub const APPLY_PERMS_VARIANT_NAME: &'static str = "apply_perms";
}
impl From<String> for command::Kind {
    fn from(value: String) -> Self {
        Self::Print(value)
    }
}
impl From<super::super::tools::command::command::Status> for command::Kind {
    fn from(value: super::super::tools::command::command::Status) -> Self {
        Self::Status(value)
    }
}
impl From<super::super::tools::perms::Perms> for command::Kind {
    fn from(value: super::super::tools::perms::Perms) -> Self {
        Self::ApplyPerms(value)
    }
}
impl command::Kind {
    pub fn get_variant_name(&self) -> &'static str {
        match self {
            Self::Status(_) => Self::STATUS_VARIANT_NAME,
            Self::Print(_) => Self::PRINT_VARIANT_NAME,
            Self::ApplyPerms(_) => Self::APPLY_PERMS_VARIANT_NAME,
        }
    }
}
///Implement [`prost::Name`] for `Command.Status`.
impl ::prost::Name for command::Status {
    const NAME: &'static str = "Command.Status";
    const PACKAGE: &'static str = "tools.command";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl command::Status {
    pub const TYPE_URL: &'static str = "/Command.Status";
}
impl command::Status {
    pub const OK_FIELD_NAME: &'static str = "ok";
    pub const ERROR_FIELD_NAME: &'static str = "error";
}
impl command::Status {
    pub const KIND_ONEOF_NAME: &'static str = "kind";
}
impl command::status::Kind {
    pub const OK_VARIANT_NAME: &'static str = "ok";
    pub const ERROR_VARIANT_NAME: &'static str = "error";
}
impl From<String> for command::status::Kind {
    fn from(value: String) -> Self {
        Self::Error(value)
    }
}
/// From source variant type to owner message type.
impl From<String> for command::Status {
    fn from(value: String) -> Self {
        Self { kind: Some(value.into()) }
    }
}
impl From<bool> for command::status::Kind {
    fn from(value: bool) -> Self {
        Self::Ok(value)
    }
}
/// From source variant type to owner message type.
impl From<bool> for command::Status {
    fn from(value: bool) -> Self {
        Self { kind: Some(value.into()) }
    }
}
impl command::status::Kind {
    pub fn get_variant_name(&self) -> &'static str {
        match self {
            Self::Ok(_) => Self::OK_VARIANT_NAME,
            Self::Error(_) => Self::ERROR_VARIANT_NAME,
        }
    }
}
/// From oneof type to owner message type.
impl From<command::status::Kind> for command::Status {
    fn from(value: command::status::Kind) -> Self {
        Self { kind: Some(value) }
    }
}
