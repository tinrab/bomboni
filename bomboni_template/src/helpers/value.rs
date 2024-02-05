use super::utility::{get_hash_opt, get_param, get_param_value};
use bomboni_proto::serde::helpers::is_truthy;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    ScopedJson,
};
use serde_json::Value;
use std::collections::BTreeMap;

pub const OBJECT_HELPER_NAME: &str = "object";
pub const OBJECT_HAS_KEY_HELPER_NAME: &str = "objectHasKey";
pub const ARRAY_HELPER_NAME: &str = "array";
pub const GROUP_BY_HELPER_NAME: &str = "groupBy";
pub const CONTAINS_HELPER_NAME: &str = "contains";
pub const NONE_HELPER_NAME: &str = "none";
pub const ALL_HELPER_NAME: &str = "all";
pub const SOME_HELPER_NAME: &str = "some";
pub const FILTER_HELPER_NAME: &str = "filter";
pub const OR_ELSE_HELPER_NAME: &str = "orElse";
pub const AND_THEN_HELPER_NAME: &str = "andThen";
pub const EITHER_OR_HELPER_NAME: &str = "eitherOr";

pub fn register_value_helpers(handlebars_registry: &mut Handlebars) {
    register_value_helpers_with_name_map(handlebars_registry, BTreeMap::default());
}

pub fn register_value_helpers_with_name_map(
    handlebars_registry: &mut Handlebars,
    name_map: BTreeMap<String, String>,
) {
    macro_rules! register_value_helper {
        ($($name:ident),* $(,)?) => {
            $(
                handlebars_registry.register_helper(
                    name_map
                    .get($name)
                    .map(String::as_str)
                    .unwrap_or($name),
                    Box::new(ValueHelper)
                );
            )*
        };
    }
    register_value_helper!(
        OBJECT_HELPER_NAME,
        OBJECT_HAS_KEY_HELPER_NAME,
        ARRAY_HELPER_NAME,
        GROUP_BY_HELPER_NAME,
        CONTAINS_HELPER_NAME,
        NONE_HELPER_NAME,
        ALL_HELPER_NAME,
        SOME_HELPER_NAME,
        FILTER_HELPER_NAME,
        OR_ELSE_HELPER_NAME,
        AND_THEN_HELPER_NAME,
        EITHER_OR_HELPER_NAME,
    );
}

#[derive(Clone, Copy)]
struct ValueHelper;

impl HelperDef for ValueHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        match h.name() {
            OBJECT_HELPER_NAME => {
                let obj: BTreeMap<_, _> = h.hash().iter().map(|(k, v)| (k, v.value())).collect();
                Ok(serde_json::to_value(obj).unwrap().into())
            }
            OBJECT_HAS_KEY_HELPER_NAME => {
                let obj = get_param_value(h, 0, "obj")?;
                let key: String = get_param(h, 1, "key")?;
                if let Value::Object(obj) = obj {
                    Ok(Value::Bool(obj.contains_key(&key)).into())
                } else {
                    Ok(Value::Bool(false).into())
                }
            }
            ARRAY_HELPER_NAME => {
                let arr: Vec<_> = h
                    .params()
                    .iter()
                    .map(handlebars::PathAndJson::value)
                    .collect();
                Ok(serde_json::to_value(arr).unwrap().into())
            }
            GROUP_BY_HELPER_NAME => {
                let values = get_param_value(h, 0, "value")?.as_array().ok_or_else(|| {
                    RenderErrorReason::ParamTypeMismatchForName(
                        GROUP_BY_HELPER_NAME,
                        "value".into(),
                        "array".into(),
                    )
                })?;
                let key: String = get_param(h, 1, "key")?;

                let groups: BTreeMap<String, Vec<&Value>> =
                    values
                        .iter()
                        .fold(BTreeMap::default(), |mut groups, value| {
                            if let Some(value_key) = value.get(&key) {
                                let key_str = if let Some(s) = value_key.as_str() {
                                    s.to_string()
                                } else {
                                    value_key.to_string()
                                };
                                groups.entry(key_str).or_default().push(value);
                            }
                            groups
                        });

                Ok(serde_json::to_value(groups).unwrap().into())
            }
            CONTAINS_HELPER_NAME => {
                let haystack = get_param_value(h, 0, "haystack")?;
                let needle = get_param_value(h, 1, "needle")?;
                Ok(serde_json::to_value(match haystack {
                    Value::String(haystack) => {
                        if let Value::String(needle) = needle {
                            haystack.contains(needle)
                        } else {
                            false
                        }
                    }
                    Value::Array(haystack) => haystack.contains(needle),
                    Value::Object(haystack) => {
                        if let Value::String(needle) = needle {
                            haystack.contains_key(needle)
                        } else {
                            false
                        }
                    }
                    _ => false,
                })
                .unwrap()
                .into())
            }
            NONE_HELPER_NAME => {
                let include_zero: bool = get_hash_opt(h, "includeZero")?.unwrap_or_default();
                for param in h.params() {
                    if is_truthy(param.value(), include_zero) {
                        return Ok(Value::Bool(false).into());
                    }
                }
                Ok(Value::Bool(true).into())
            }
            ALL_HELPER_NAME => {
                let include_zero: bool = get_hash_opt(h, "includeZero")?.unwrap_or_default();
                for param in h.params() {
                    if !is_truthy(param.value(), include_zero) {
                        return Ok(Value::Bool(false).into());
                    }
                }
                Ok(Value::Bool(true).into())
            }
            SOME_HELPER_NAME => {
                let include_zero: bool = get_hash_opt(h, "includeZero")?.unwrap_or_default();
                for param in h.params() {
                    if is_truthy(param.value(), include_zero) {
                        return Ok(Value::Bool(true).into());
                    }
                }
                Ok(Value::Bool(false).into())
            }
            FILTER_HELPER_NAME => {
                let haystack = get_param_value(h, 0, "haystack")?;
                let predicate: String = get_param(h, 1, "predicate")?;
                let include_zero: bool = get_hash_opt(h, "includeZero")?.unwrap_or_default();
                match haystack {
                    Value::Array(haystack) => {
                        let mut selected = Vec::new();
                        for item in haystack {
                            let rendered = r.render_template(&predicate, item)?;
                            let result: Value =
                                serde_json::from_str(&rendered).unwrap_or(Value::Bool(false));
                            if is_truthy(&result, include_zero) {
                                selected.push(item.clone());
                            }
                        }
                        Ok(Value::Array(selected).into())
                    }
                    _ => Ok(haystack.clone().into()),
                }
            }
            OR_ELSE_HELPER_NAME => {
                let item = get_param_value(h, 0, "item")?;
                let fallback = get_param_value(h, 1, "fallback")?;
                let include_zero: bool = get_hash_opt(h, "includeZero")?.unwrap_or_default();
                Ok(if is_truthy(item, include_zero) {
                    item.clone().into()
                } else {
                    fallback.clone().into()
                })
            }
            AND_THEN_HELPER_NAME => {
                let item = get_param_value(h, 0, "item")?;
                let fallback = get_param_value(h, 1, "fallback")?;
                let include_zero: bool = get_hash_opt(h, "includeZero")?.unwrap_or_default();
                Ok(if is_truthy(item, include_zero) {
                    fallback.clone().into()
                } else {
                    item.clone().into()
                })
            }
            EITHER_OR_HELPER_NAME => {
                let condition = get_param_value(h, 0, "condition")?;
                let left = get_param_value(h, 1, "left")?;
                let right = get_param_value(h, 2, "right")?;
                let include_zero: bool = get_hash_opt(h, "includeZero")?.unwrap_or_default();
                Ok(if is_truthy(condition, include_zero) {
                    left.clone().into()
                } else {
                    right.clone().into()
                })
            }
            _ => unreachable!("helper `{}` is not implemented", h.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::string::register_string_helpers;

    use super::*;

    #[test]
    fn object_helpers() {
        let r = get_handlebars_registry();

        assert_eq!(
            r.render_template(r#"{{objectHasKey (object x=42) "x"}}"#, &Value::Null)
                .unwrap()
                .as_str(),
            "true"
        );
    }

    #[test]
    fn array_helpers() {
        let r = get_handlebars_registry();

        assert_eq!(
            r.render_template(
                r#"{{toJson (groupBy (array (object x=1 y=2) (object x=2 y=4)) "x")}}"#,
                &Value::Null
            )
            .unwrap(),
            r#"{"1":[{"x":1,"y":2}],"2":[{"x":2,"y":4}]}"#
        );

        assert_eq!(
            r.render_template(r"{{contains (array 1 2 3 4) 2}}", &Value::Null)
                .unwrap()
                .as_str(),
            "true"
        );
        assert_eq!(
            r.render_template(r"{{contains (array 1 3 4) 2}}", &Value::Null)
                .unwrap()
                .as_str(),
            "false"
        );

        assert_eq!(
            r.render_template(r"{{none false 0}}", &Value::Null)
                .unwrap()
                .as_str(),
            "true"
        );
    }

    fn get_handlebars_registry() -> Handlebars<'static> {
        let mut r = Handlebars::new();
        r.set_strict_mode(true);
        register_value_helpers(&mut r);
        register_string_helpers(&mut r);
        r
    }
}
