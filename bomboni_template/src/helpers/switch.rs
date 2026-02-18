//! Handlebars helper for switch statements.
//!
//! Inspired by [handlebars_switch][1] with support for multi-value match cases.
//!
//! [1]: https://github.com/nickjer/handlebars_switch

use crate::helpers::utility::get_param_value;
use handlebars::{
    BlockContext, Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext,
    Renderable,
};
use serde_json::Value;

/// Name of the switch helper.
pub const SWITCH_HELPER_NAME: &str = "switch";
/// Name of the case helper.
pub const SWITCH_CASE_HELPER_NAME: &str = "case";
/// Name of the default helper.
pub const SWITCH_DEFAULT_HELPER_NAME: &str = "default";

const MATCH_LOCAL_NAME: &str = "__match";
const CASE_VALUE_LOCAL_NAME: &str = "value";

/// Registers the switch helper with Handlebars registry.
pub fn register_switch_helper(handlebars_registry: &mut Handlebars) {
    handlebars_registry.register_helper(SWITCH_HELPER_NAME, Box::new(switch_helper));
}

/// Switch helper for handlebars.
///
/// # Errors
///
/// Will return [`HelperResult`] if template rendering fails or parameter retrieval fails.
pub fn switch_helper(
    h: &Helper,
    r: &Handlebars,
    ctx: &Context,
    rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = get_param_value(h, 0, "param")?;

    let mut block_context = BlockContext::new();
    block_context.set_local_var(MATCH_LOCAL_NAME, Value::Bool(false));
    let mut local_rc = rc.clone();
    local_rc.push_block(block_context);

    local_rc.register_local_helper(
        SWITCH_CASE_HELPER_NAME,
        Box::new(CaseHelper {
            switch_value: param,
        }),
    );
    local_rc.register_local_helper(
        SWITCH_DEFAULT_HELPER_NAME,
        Box::new(DefaultHelper {
            switch_value: param,
        }),
    );

    let result = h
        .template()
        .map_or_else(|| Ok(()), |t| t.render(r, ctx, &mut local_rc, out));

    local_rc.pop_block();

    result
}

struct CaseHelper<'a> {
    switch_value: &'a Value,
}

impl HelperDef for CaseHelper<'_> {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        if let Some(ref mut block) = rc.block_mut() {
            if block
                .get_local_var(MATCH_LOCAL_NAME)
                .and_then(Value::as_bool)
                .unwrap_or_default()
            {
                return Ok(());
            }
            for param in h.params() {
                if param.value() == self.switch_value {
                    block.set_local_var(MATCH_LOCAL_NAME, Value::Bool(true));
                    block.set_local_var(CASE_VALUE_LOCAL_NAME, param.value().clone());
                    h.template()
                        .map_or_else(|| Ok(()), |t| t.render(r, ctx, rc, out))?;
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}

struct DefaultHelper<'a> {
    switch_value: &'a Value,
}

impl HelperDef for DefaultHelper<'_> {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        if let Some(block) = rc.block_mut() {
            if block
                .get_local_var(MATCH_LOCAL_NAME)
                .and_then(Value::as_bool)
                .unwrap_or_default()
            {
                return Ok(());
            }
            // Include @value local variable even if it doesn't match any case
            block.set_local_var(CASE_VALUE_LOCAL_NAME, self.switch_value.clone());
            h.template()
                .map_or_else(|| Ok(()), |t| t.render(r, ctx, rc, out))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::string::register_string_helpers;

    use super::*;
    use serde_json::json;

    #[test]
    fn it_works() {
        let r = get_handlebars();
        let t = r"
            {{#switch x}}
                {{#case 42}}life{{/case}}
                {{#case 1337}}leet{{/case}}
                {{#default}}number{{/default}}
            {{/switch}}
        ";
        assert_eq!(
            r.render_template(t, &json!({"x": 42})).unwrap().trim(),
            "life"
        );
        assert_eq!(
            r.render_template(t, &json!({"x": 1337})).unwrap().trim(),
            "leet"
        );
        assert_eq!(
            r.render_template(t, &json!({"x": 2})).unwrap().trim(),
            "number"
        );
    }

    #[test]
    fn multi_value() {
        let r = get_handlebars();
        let t = r#"
            {{#switch x}}
                {{#case 42}}life{{/case}}
                {{#case 2 4 8}}
                    power of two ({{toString @value}})
                {{/case}}
                {{#case "foo" "bar" "baz"}}funny word ({{@value}}){{/case}}
                {{#default}}number: {{@value}}{{/default}}
            {{/switch}}
        "#;
        assert_eq!(
            r.render_template(t, &json!({"x": 42})).unwrap().trim(),
            "life"
        );
        assert_eq!(
            r.render_template(t, &json!({"x": 2})).unwrap().trim(),
            "power of two (2)"
        );
        assert_eq!(
            r.render_template(t, &json!({"x": 8})).unwrap().trim(),
            "power of two (8)"
        );
        assert_ne!(
            r.render_template(t, &json!({"x": 5})).unwrap().trim(),
            "power of two (8)"
        );
        assert_eq!(
            r.render_template(t, &json!({"x": "baz"})).unwrap().trim(),
            "funny word (baz)"
        );
        assert_eq!(
            r.render_template(t, &json!({"x": 1337})).unwrap().trim(),
            "number: 1337"
        );
        assert_eq!(
            r.render_template(t, &json!({"x": 13})).unwrap().trim(),
            "number: 13"
        );
    }

    #[test]
    fn nested_match() {
        let r = get_handlebars();

        let t = r#"
        {{~#switch s~}}
            {{~#case "i32" "i64"~}}
                signed ({{toString @value}}):
                {{~#switch b~}}
                    {{~#case 1 2 4~}}
                        s, b => {{@../value}} {{@value}}
                    {{~/case~}}
                    {{~#default~}}
                        None
                    {{~/default~}}
                {{~/switch~}}
            {{~/case~}}
            {{~#case "u32" "u64"~}}
                unsigned ({{toString @value}})
            {{~/case~}}
        {{~/switch~}}
    "#;
        assert_eq!(
            r.render_template(t, &json!({"b": 4, "s": "i32"}))
                .unwrap()
                .trim(),
            "signed (i32):s, b => i32 4"
        );
    }

    fn get_handlebars() -> Handlebars<'static> {
        let mut r = Handlebars::new();
        r.set_strict_mode(true);
        register_switch_helper(&mut r);
        register_string_helpers(&mut r);
        r
    }
}
