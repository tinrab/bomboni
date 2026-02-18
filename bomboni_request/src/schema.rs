use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use crate::value::Value;

/// Schema for query validation.
#[derive(Debug, Clone)]
pub struct Schema {
    /// Schema members.
    pub members: BTreeMap<String, MemberSchema>,
}

/// Schema member type.
#[derive(Debug, Clone, PartialEq)]
pub enum MemberSchema {
    /// Resource member.
    Resource(ResourceMemberSchema),
    /// Field member.
    Field(FieldMemberSchema),
}

/// Resource member schema.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceMemberSchema {
    /// Resource fields.
    pub fields: BTreeMap<String, MemberSchema>,
}

/// Field member schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldMemberSchema {
    /// Value type.
    pub value_type: ValueType,
    /// Whether field is repeated.
    pub repeated: bool,
    /// Whether field is ordered.
    pub ordered: bool,
    /// Whether field allows has operator.
    pub allow_has_operator: bool,
}

/// Function schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSchema {
    /// Argument value types.
    pub argument_value_types: Vec<ValueType>,
    /// Return value type.
    pub return_value_type: ValueType,
}

/// Map of function schemas.
pub type FunctionSchemaMap = BTreeMap<String, FunctionSchema>;

/// Value type for schema validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    /// Integer value.
    Integer,
    /// Float value.
    Float,
    /// Boolean value.
    Boolean,
    /// String value.
    String,
    /// Timestamp value.
    Timestamp,
    /// Any value.
    Any,
    // ResourceName,
}

/// Trait for types that can be mapped to schema values.
pub trait SchemaMapped {
    /// Gets field value by name.
    fn get_field(&self, name: &str) -> Value;
}

impl Schema {
    /// Gets member schema by name.
    pub fn get_member(&self, name: &str) -> Option<&MemberSchema> {
        let mut member: Option<&MemberSchema> = None;
        for step in name.split('.') {
            if let Some(upper_member) = member {
                if let MemberSchema::Resource(resource) = upper_member {
                    if let Some(resource_field) = resource.fields.get(step) {
                        member = Some(resource_field);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else if let Some(step_member) = self.members.get(step) {
                member = Some(step_member);
            } else {
                return None;
            }
        }
        member
    }

    /// Gets field schema by name.
    pub fn get_field(&self, name: &str) -> Option<&FieldMemberSchema> {
        if let Some(MemberSchema::Field(field)) = self.get_member(name) {
            Some(field)
        } else {
            None
        }
    }
}

impl FieldMemberSchema {
    /// Creates a new field member schema.
    pub const fn new(value_type: ValueType) -> Self {
        Self {
            value_type,
            repeated: false,
            ordered: false,
            allow_has_operator: true,
        }
    }

    /// Creates a new ordered field member schema.
    pub const fn new_ordered(value_type: ValueType) -> Self {
        Self {
            value_type,
            repeated: false,
            ordered: true,
            allow_has_operator: true,
        }
    }

    /// Creates a new repeated field member schema.
    pub const fn new_repeated(value_type: ValueType) -> Self {
        Self {
            value_type,
            repeated: true,
            ordered: false,
            allow_has_operator: true,
        }
    }
}

impl From<FieldMemberSchema> for MemberSchema {
    fn from(field: FieldMemberSchema) -> Self {
        Self::Field(field)
    }
}

impl From<ResourceMemberSchema> for MemberSchema {
    fn from(resource: ResourceMemberSchema) -> Self {
        Self::Resource(resource)
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::schema::RequestItem;

    use super::*;

    #[test]
    fn get_member() {
        let schema = RequestItem::get_schema();
        assert!(matches!(
            schema.get_member("user"),
            Some(MemberSchema::Resource(_))
        ));
        assert!(matches!(
            schema.get_member("task.deleted"),
            Some(MemberSchema::Field(field)) if field.value_type == ValueType::Boolean
        ));
        assert!(schema.get_field("user.id").unwrap().ordered);
        assert!(schema.get_field("task.tags").unwrap().repeated);
    }
}
