use super::utility::get_param;
use handlebars::{
    Context, Decorator, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError,
};
use std::collections::BTreeMap;

pub const RENDER_HELPER_NAME: &str = "render";
pub const SET_DECORATOR_NAME: &str = "set";

pub fn register_render_helpers(handlebars_registry: &mut Handlebars) {
    register_render_helpers_with_name_map(handlebars_registry, BTreeMap::default());
}

pub fn register_render_helpers_with_name_map(
    handlebars_registry: &mut Handlebars,
    name_map: BTreeMap<String, String>,
) {
    macro_rules! name {
        ($name:expr) => {
            name_map.get($name).map(String::as_str).unwrap_or($name)
        };
    }
    handlebars_registry.register_helper(name!(RENDER_HELPER_NAME), Box::new(render_helper));
    handlebars_registry.register_decorator(name!(SET_DECORATOR_NAME), Box::new(set_decorator));
}

fn render_helper(
    h: &Helper,
    r: &Handlebars,
    ctx: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let template: String = get_param(h, 0, "template")?;

    let rendered = r.render_template_with_context(&template, ctx)?;
    out.write(&rendered)?;

    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn set_decorator(
    d: &Decorator,
    _: &Handlebars,
    _: &Context,
    rc: &mut RenderContext,
) -> Result<(), RenderError> {
    if let Some(block) = rc.block_mut() {
        for (k, v) in d.hash() {
            block.set_local_var(k, v.value().clone());
        }
    } else {
        unreachable!("block not set")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::helpers::string::register_string_helpers;
    use serde_json::{json, Value};

    use super::*;

    #[test]
    fn it_works() {
        let mut r = Handlebars::new();
        r.set_strict_mode(true);
        register_render_helpers(&mut r);
        register_string_helpers(&mut r);

        r.register_template_string("title", r"{{titleCase title}}!")
            .unwrap();

        assert_eq!(
            r.render_template(r#"{{render "x = {{x}}"}}"#, &json!({"x": 42}))
                .unwrap(),
            "x = 42"
        );
        assert_eq!(
            r.render_template(
                r#"
                    {{*set name = "John" age = 14000}}
                    {{@name}} is {{@age}} years old.
                "#,
                &Value::Null
            )
            .unwrap()
            .trim(),
            "John is 14000 years old."
        );
    }
}
