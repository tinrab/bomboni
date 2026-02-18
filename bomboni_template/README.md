# bomboni_template

Utilities for working with Handlebars templates.

This crate provides a collection of custom Handlebars helpers for template rendering.
It includes helpers for mathematical operations, string transformations, value manipulation, conditional logic, and dynamic template rendering.

## Examples

### Math Operations

```rust
use handlebars::Handlebars;
use serde_json::Value;
use bomboni_template::helpers::register_helpers;

let mut hbs = Handlebars::new();
register_helpers(&mut hbs);

// Basic arithmetic
assert_eq!(hbs.render_template("{{add 1 2}}", &Value::Null).unwrap(), "3.0");

// Advanced operations
assert_eq!(hbs.render_template("{{sqrt 2}}", &Value::Null).unwrap(), "1.4142135623730951");
assert_eq!(hbs.render_template("{{pow 2 8}}", &Value::Null).unwrap(), "256.0");
assert_eq!(hbs.render_template("{{clamp 5 1 3}}", &Value::Null).unwrap(), "3.0");
```

### String Case Conversion

```rust
use handlebars::Handlebars;
use serde_json::Value;
use bomboni_template::helpers::register_helpers;

let mut hbs = Handlebars::new();
register_helpers(&mut hbs);

// Case conversion
assert_eq!(
    hbs.render_template(r#"{{upperCase "hello world"}}"#, &Value::Null).unwrap(),
    "HELLO WORLD"
);

assert_eq!(
    hbs.render_template(r#"{{pascalCase "hello world"}}"#, &Value::Null).unwrap(),
    "HelloWorld"
);

assert_eq!(
    hbs.render_template(r#"{{snakeCase "hello world"}}"#, &Value::Null).unwrap(),
    "hello_world"
);

assert_eq!(
    hbs.render_template(r#"{{screamingSnakeCase "hello world"}}"#, &Value::Null).unwrap(),
    "HELLO_WORLD"
);

// String formatting
assert_eq!(
    hbs.render_template(r"{{toIntegerString 3.14}}", &Value::Null).unwrap(),
    "3"
);
```

### Value Manipulation

```rust
use handlebars::Handlebars;
use serde_json::{Value, json};
use bomboni_template::helpers::register_helpers;

let mut hbs = Handlebars::new();
register_helpers(&mut hbs);

// Object operations
assert_eq!(
    hbs.render_template(r#"{{objectHasKey (object x=42) "x"}}"#, &Value::Null).unwrap(),
    "true"
);

// Array operations
assert_eq!(
    hbs.render_template(r"{{contains (array 1 2 3 4) 2}}", &Value::Null).unwrap(),
    "true"
);

// Check truthiness
assert_eq!(
    hbs.render_template(r"{{none false 0}}", &Value::Null).unwrap(),
    "true"
);

// Group by key
hbs.render_template(
    r#"{{groupBy (array (object x=1 y=2) (object x=2 y=4)) "x"}}"#,
    &Value::Null
).unwrap();
```

### Switch/Case Pattern Matching

```rust
use handlebars::Handlebars;
use serde_json::json;
use bomboni_template::helpers::register_helpers;

let mut hbs = Handlebars::new();
register_helpers(&mut hbs);

let template = r#"
{{#switch x}}
    {{#case 42}}life{{/case}}
    {{#case 2 4 8}}power of two{{/case}}
    {{#default}}other{{/default}}
{{/switch}}
"#;

assert_eq!(hbs.render_template(template, &json!({"x": 42})).unwrap().trim(), "life");
assert_eq!(hbs.render_template(template, &json!({"x": 2})).unwrap().trim(), "power of two");
assert_eq!(hbs.render_template(template, &json!({"x": 5})).unwrap().trim(), "other");
```

### Dynamic Template Rendering and Local Variables

```rust
use handlebars::Handlebars;
use serde_json::json;
use bomboni_template::helpers::register_helpers;

let mut hbs = Handlebars::new();
register_helpers(&mut hbs);

// Dynamic template rendering
assert_eq!(
    hbs.render_template(r#"{{render "x = {{x}}"}}"#, &json!({"x": 42})).unwrap(),
    "x = 42"
);

// Set local variables
let template = r#"
{{*set name = "John" age = 14000}}
{{@name}} is {{@age}} years old.
"#;
let rendered = hbs.render_template(template, &serde_json::Value::Null).unwrap();
assert_eq!(rendered.trim(), "John is 14000 years old.");
```

## Helpers Reference

### Math Helpers

- `add`: Addition
- `subtract`: Subtraction
- `multiply`: Multiplication
- `divide`: Division
- `modulo`: Modulo
- `negate`: Negate value
- `absolute`: Absolute value
- `round`: Round to nearest integer
- `ceil`: Ceiling
- `floor`: Floor
- `sqrt`: Square root
- `sign`: Sign of number
- `pow`: Power (x^p)
- `clamp`: Clamp value between min and max

### String Helpers

**Case Conversion:**
- `upperCase`: UPPER CASE
- `lowerCase`: lower case
- `titleCase`: Title Case
- `camelCase`: camelCase
- `pascalCase`: PascalCase
- `snakeCase`: snake_case
- `screamingSnakeCase`: SCREAMING_SNAKE_CASE
- `kebabCase`: kebab-case
- `trainCase`: Train-Case
- And many more (toggleCase, alternatingCase, cobolCase, flatCase, etc.)

**Formatting:**
- `toString`: Convert value to string
- `toJson`: Convert to JSON (with optional `pretty` parameter)
- `concat`: Concatenate multiple values
- `toIntegerString`: Convert to integer string

### Value Helpers

- `object`: Create object from named parameters
- `objectHasKey`: Check if object has key
- `array`: Create array from parameters
- `groupBy`: Group array by key
- `contains`: Check if haystack contains needle
- `none`: Check if all values are falsy
- `all`: Check if all values are truthy
- `some`: Check if some value is truthy
- `filter`: Filter array by predicate template
- `orElse`: Return item if truthy, else fallback
- `andThen`: Return fallback if item truthy, else item
- `eitherOr`: Conditional value selection

### Switch Helper

- `switch`: Begin switch statement (takes value parameter)
- `case`: Match one or more values (supports multiple value parameters)
- `default`: Fallback when no case matches

### Render and Decorators

- `render`: Dynamically render template string
- `set`: Set local variables (decorator syntax)
