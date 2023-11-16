use std::{collections::BTreeMap, ops::Neg};

use handlebars::{Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, ScopedJson};
use serde_json::Value;

use super::utility::get_param_value;

pub const ADD_HELPER_NAME: &str = "add";
pub const SUBTRACT_HELPER_NAME: &str = "subtract";
pub const MULTIPLY_HELPER_NAME: &str = "multiply";
pub const DIVIDE_HELPER_NAME: &str = "divide";
pub const MODULO_HELPER_NAME: &str = "modulo";

pub const NEGATE_HELPER_NAME: &str = "negate";
pub const ABSOLUTE_HELPER_NAME: &str = "absolute";
pub const ROUND_HELPER_NAME: &str = "round";
pub const CEIL_HELPER_NAME: &str = "ceil";
pub const FLOOR_HELPER_NAME: &str = "floor";
pub const SQRT_HELPER_NAME: &str = "sqrt";
pub const SIGN_HELPER_NAME: &str = "sign";

pub const POW_HELPER_NAME: &str = "pow";
pub const CLAMP_HELPER_NAME: &str = "clamp";

pub fn register_math_helpers(handlebars_registry: &mut Handlebars) {
    register_math_helpers_with_name_map(handlebars_registry, BTreeMap::default());
}

pub fn register_math_helpers_with_name_map(
    handlebars_registry: &mut Handlebars,
    name_map: BTreeMap<String, String>,
) {
    macro_rules! register_helpers {
        ($($name:ident),* $(,)?) => {
            $(
                handlebars_registry.register_helper(
                    name_map.get($name).map(String::as_str)
                        .unwrap_or($name),
                    Box::new(MathHelper),
                );
            )*
        };
    }

    register_helpers!(
        ADD_HELPER_NAME,
        SUBTRACT_HELPER_NAME,
        MULTIPLY_HELPER_NAME,
        DIVIDE_HELPER_NAME,
        MODULO_HELPER_NAME,
        NEGATE_HELPER_NAME,
        ABSOLUTE_HELPER_NAME,
        ROUND_HELPER_NAME,
        CEIL_HELPER_NAME,
        FLOOR_HELPER_NAME,
        SQRT_HELPER_NAME,
        SIGN_HELPER_NAME,
        POW_HELPER_NAME,
        CLAMP_HELPER_NAME,
    );
}

#[derive(Clone, Copy)]
struct MathHelper;

impl HelperDef for MathHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        macro_rules! binary_op {
            ($op:tt) => {{
                let a: f64 = get_param_value(h, 0, "a")?.as_f64().unwrap_or(0.0);
                let b: f64 = get_param_value(h, 1, "b")?.as_f64().unwrap_or(0.0);
                Ok(Value::from((a + b)).into())
            }};
        }
        macro_rules! unary_op {
            ($op:tt) => {{
                let x: f64 = get_param_value(h, 0, "x")?.as_f64().unwrap_or(0.0);
                Ok(Value::from(x.$op()).into())
            }};
        }
        match h.name() {
            ADD_HELPER_NAME => binary_op!(+),
            SUBTRACT_HELPER_NAME => binary_op!(-),
            MULTIPLY_HELPER_NAME => binary_op!(*),
            DIVIDE_HELPER_NAME => binary_op!(/),
            MODULO_HELPER_NAME => binary_op!(%),
            NEGATE_HELPER_NAME => unary_op!(neg),
            ABSOLUTE_HELPER_NAME => unary_op!(abs),
            ROUND_HELPER_NAME => unary_op!(round),
            CEIL_HELPER_NAME => unary_op!(ceil),
            FLOOR_HELPER_NAME => unary_op!(floor),
            SQRT_HELPER_NAME => unary_op!(sqrt),
            SIGN_HELPER_NAME => unary_op!(signum),
            POW_HELPER_NAME => {
                let x: f64 = get_param_value(h, 0, "x")?.as_f64().unwrap_or(0.0);
                let p: f64 = get_param_value(h, 1, "p")?.as_f64().unwrap_or(0.0);
                Ok(Value::from(x.powf(p)).into())
            }
            CLAMP_HELPER_NAME => {
                let x: f64 = get_param_value(h, 0, "x")?.as_f64().unwrap_or(0.0);
                let min: f64 = get_param_value(h, 1, "min")?.as_f64().unwrap_or(0.0);
                let max: f64 = get_param_value(h, 2, "max")?.as_f64().unwrap_or(0.0);
                Ok(Value::from(x.max(min).min(max)).into())
            }
            _ => unreachable!("helper `{}` is not implemented", h.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let r = get_handlebars_registry();
        macro_rules! assert_expr {
            ($expr:expr, $result:expr) => {
                assert_eq!(r.render_template($expr, &Value::Null).unwrap(), $result);
            };
        }

        assert_expr!("{{add 1 2}}", "3.0");
        assert_expr!("{{sqrt 2}}", "1.4142135623730951");
        assert_expr!("{{pow 2 8}}", "256.0");
        assert_expr!("{{clamp 5 1 3}}", "3.0");
    }

    fn get_handlebars_registry() -> Handlebars<'static> {
        let mut r = Handlebars::new();
        r.set_strict_mode(true);
        register_math_helpers(&mut r);
        r
    }
}
