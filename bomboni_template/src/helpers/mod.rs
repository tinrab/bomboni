use handlebars::Handlebars;

pub mod math;
pub mod render;
pub mod string;
pub mod switch;
pub mod utility;
pub mod value;

pub fn register_helpers(handlebars_registry: &mut Handlebars) {
    math::register_math_helpers(handlebars_registry);
    render::register_render_helpers(handlebars_registry);
    string::register_string_helpers(handlebars_registry);
    value::register_value_helpers(handlebars_registry);
    switch::register_switch_helper(handlebars_registry);
}
