use crate::helpers::utility::get_param_opt;
use crate::helpers::utility::get_param_value;
use convert_case::{Case, Casing};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use std::collections::BTreeMap;

use super::utility::get_param;

pub const UPPER_CASE_HELPER_NAME: &str = "upperCase";
pub const LOWER_CASE_HELPER_NAME: &str = "lowerCase";
pub const TITLE_CASE_HELPER_NAME: &str = "titleCase";
pub const TOGGLE_CASE_HELPER_NAME: &str = "toggleCase";
pub const ALTERNATING_CASE_HELPER_NAME: &str = "alternatingCase";
pub const CAMEL_CASE_HELPER_NAME: &str = "camelCase";
pub const PASCAL_CASE_HELPER_NAME: &str = "pascalCase";
pub const UPPER_CAMEL_CASE_HELPER_NAME: &str = "upperCamelCase";
pub const SNAKE_CASE_HELPER_NAME: &str = "snakeCase";
pub const UPPER_SNAKE_CASE_HELPER_NAME: &str = "upperSnakeCase";
pub const SCREAMING_SNAKE_CASE_HELPER_NAME: &str = "screamingSnakeCase";
pub const KEBAB_CASE_HELPER_NAME: &str = "kebabCase";
pub const COBOL_CASE_HELPER_NAME: &str = "cobolCase";
pub const TRAIN_CASE_HELPER_NAME: &str = "trainCase";
pub const FLAT_CASE_HELPER_NAME: &str = "flatCase";
pub const UPPER_FLAT_CASE_HELPER_NAME: &str = "upperFlatCase";

pub const TO_STRING_HELPER_NAME: &str = "toString";
pub const TO_JSON_HELPER_NAME: &str = "toJson";
pub const CONCAT_HELPER_NAME: &str = "concat";
pub const TO_INTEGER_STRING_HELPER_NAME: &str = "toIntegerString";

pub fn register_string_helpers(handlebars_registry: &mut Handlebars) {
    register_string_helpers_with_name_map(handlebars_registry, BTreeMap::default());
}

pub fn register_string_helpers_with_name_map(
    handlebars_registry: &mut Handlebars,
    name_map: BTreeMap<String, String>,
) {
    macro_rules! name {
        ($name:expr) => {
            name_map.get($name).map(String::as_str).unwrap_or($name)
        };
    }
    macro_rules! register_case_helper {
        ($($name:ident),* $(,)?) => {
            $(
                handlebars_registry.register_helper(
                    name!($name),
                    Box::new(convert_case_helper),
                );
            )*
        };
    }
    register_case_helper!(
        UPPER_CASE_HELPER_NAME,
        LOWER_CASE_HELPER_NAME,
        TITLE_CASE_HELPER_NAME,
        TOGGLE_CASE_HELPER_NAME,
        ALTERNATING_CASE_HELPER_NAME,
        CAMEL_CASE_HELPER_NAME,
        PASCAL_CASE_HELPER_NAME,
        UPPER_CAMEL_CASE_HELPER_NAME,
        SNAKE_CASE_HELPER_NAME,
        UPPER_SNAKE_CASE_HELPER_NAME,
        SCREAMING_SNAKE_CASE_HELPER_NAME,
        KEBAB_CASE_HELPER_NAME,
        COBOL_CASE_HELPER_NAME,
        TRAIN_CASE_HELPER_NAME,
        FLAT_CASE_HELPER_NAME,
        UPPER_FLAT_CASE_HELPER_NAME,
    );

    handlebars_registry.register_helper(name!(TO_STRING_HELPER_NAME), Box::new(to_string_helper));
    handlebars_registry.register_helper(name!(TO_JSON_HELPER_NAME), Box::new(to_json_helper));
    handlebars_registry.register_helper(name!(CONCAT_HELPER_NAME), Box::new(concat_helper));
    handlebars_registry.register_helper(
        name!(TO_INTEGER_STRING_HELPER_NAME),
        Box::new(to_integer_string_helper),
    );
}

fn convert_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let value = get_param_value(h, 0, "value")?;
    let value = if let Some(s) = value.as_str() {
        s.to_string()
    } else {
        value.to_string()
    };
    out.write(&match h.name() {
        UPPER_CASE_HELPER_NAME => value.to_case(Case::Upper),
        LOWER_CASE_HELPER_NAME => value.to_case(Case::Lower),
        TITLE_CASE_HELPER_NAME => value.to_case(Case::Title),
        TOGGLE_CASE_HELPER_NAME => value.to_case(Case::Toggle),
        ALTERNATING_CASE_HELPER_NAME => value.to_case(Case::Alternating),
        CAMEL_CASE_HELPER_NAME => value.to_case(Case::Camel),
        PASCAL_CASE_HELPER_NAME => value.to_case(Case::Pascal),
        UPPER_CAMEL_CASE_HELPER_NAME => value.to_case(Case::UpperCamel),
        SNAKE_CASE_HELPER_NAME => value.to_case(Case::Snake),
        UPPER_SNAKE_CASE_HELPER_NAME => value.to_case(Case::UpperSnake),
        SCREAMING_SNAKE_CASE_HELPER_NAME => value.to_case(Case::ScreamingSnake),
        KEBAB_CASE_HELPER_NAME => value.to_case(Case::Kebab),
        COBOL_CASE_HELPER_NAME => value.to_case(Case::Cobol),
        TRAIN_CASE_HELPER_NAME => value.to_case(Case::Train),
        FLAT_CASE_HELPER_NAME => value.to_case(Case::Flat),
        UPPER_FLAT_CASE_HELPER_NAME => value.to_case(Case::UpperFlat),
        _ => unreachable!("helper `{}` not implemented", h.name()),
    })?;

    Ok(())
}

fn to_string_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let value = get_param_value(h, 0, "value")?;
    if let Some(s) = value.as_str() {
        out.write(s)?;
    } else {
        out.write(&value.to_string())?;
    }
    Ok(())
}

fn to_json_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let value = get_param_value(h, 0, "value")?;
    let pretty: bool = get_param_opt(h, 1, "pretty")?.unwrap_or_default();

    out.write(
        &(if pretty {
            serde_json::to_string_pretty(value)
        } else {
            serde_json::to_string(value)
        })
        .unwrap(),
    )?;

    Ok(())
}

fn concat_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let mut buf = String::new();
    for param in h.params().iter().map(handlebars::PathAndJson::value) {
        if let Some(s) = param.as_str() {
            buf.push_str(s);
        } else {
            buf.push_str(&param.to_string());
        }
    }
    out.write(&buf)?;
    Ok(())
}

fn to_integer_string_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let value: f64 = get_param(h, 0, "value")?;
    out.write(&(value.trunc() as i64).to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn convert_case() {
        let r = get_handlebars_registry();
        macro_rules! assert_case {
            ($case:expr, $source:expr, $expected:expr $(,)?) => {
                assert_eq!(
                    r.render_template(
                        &format!(r#"{{{{{} "{}" }}}}"#, $case, $source),
                        &Value::Null
                    )
                    .unwrap()
                    .as_str(),
                    $expected
                );
            };
        }

        assert_case!(UPPER_CASE_HELPER_NAME, "variable name", "VARIABLE NAME");
        assert_case!(PASCAL_CASE_HELPER_NAME, "variable name", "VariableName");
        assert_case!(
            SCREAMING_SNAKE_CASE_HELPER_NAME,
            "variable name",
            "VARIABLE_NAME"
        );
        assert_case!(CAMEL_CASE_HELPER_NAME, "variable name", "variableName");
    }

    #[test]
    fn printing() {
        let r = get_handlebars_registry();
        macro_rules! assert_print {
            ($expr:expr, $result:expr) => {
                assert_eq!(r.render_template($expr, &Value::Null).unwrap(), $result);
            };
        }

        assert_print!(r"{{toIntegerString 3.14}}", "3");
    }

    fn get_handlebars_registry() -> Handlebars<'static> {
        let mut r = Handlebars::new();
        r.set_strict_mode(true);
        register_string_helpers(&mut r);
        r
    }
}
