use handlebars::Handlebars;

/// Math helper functions.
pub mod math;
/// Render helper functions.
pub mod render;
/// String helper functions.
pub mod string;
/// Switch helper functions.
pub mod switch;
/// Utility helper functions.
pub mod utility;
/// Value helper functions.
pub mod value;

/// Registers all template helpers with the Handlebars registry.
pub fn register_helpers(handlebars_registry: &mut Handlebars) {
    math::register_math_helpers(handlebars_registry);
    render::register_render_helpers(handlebars_registry);
    string::register_string_helpers(handlebars_registry);
    value::register_value_helpers(handlebars_registry);
    switch::register_switch_helper(handlebars_registry);
}
