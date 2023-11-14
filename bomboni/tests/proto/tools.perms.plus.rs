///Implement [`prost::Name`] for `Perms`.
impl ::prost::Name for Perms {
    const NAME: &'static str = "Perms";
    const PACKAGE: &'static str = "tools.perms";
    fn full_name() -> String {
        format!("{}.{}", Self::PACKAGE, Self::NAME)
    }
    fn type_url() -> String {
        Self::TYPE_URL.into()
    }
}
impl Perms {
    pub const TYPE_URL: &'static str = "/Perms";
}
impl Perms {
    pub const CODE_FIELD_NAME: &'static str = "code";
}
