use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use crate::request::value::Value;

#[derive(Debug, Clone)]
pub struct Schema {
    pub members: BTreeMap<String, MemberSchema>,
    pub functions: BTreeMap<String, FunctionSchema>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemberSchema {
    Resource(ResourceMemberSchema),
    Field(FieldMemberSchema),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceMemberSchema {
    pub fields: BTreeMap<String, MemberSchema>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldMemberSchema {
    pub value_type: ValueType,
    pub repeated: bool,
    pub ordered: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSchema {
    pub argument_value_types: Vec<ValueType>,
    pub return_value_type: ValueType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Integer,
    Float,
    Boolean,
    String,
    Timestamp,
    Any,
}

pub trait SchemaMapped {
    fn get_field(&self, name: &str) -> Value;
}

impl Schema {
    pub fn is_ordered(&self, field: &str) -> bool {
        if let Some(member) = self.get_member(field) {
            match member {
                MemberSchema::Resource(_) => false,
                MemberSchema::Field(field) => field.ordered,
            }
        } else {
            false
        }
    }

    pub fn is_repeated(&self, field: &str) -> bool {
        if let Some(member) = self.get_member(field) {
            match member {
                MemberSchema::Resource(_) => false,
                MemberSchema::Field(field) => field.repeated,
            }
        } else {
            false
        }
    }

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
}

impl FieldMemberSchema {
    pub fn new(value_type: ValueType) -> Self {
        FieldMemberSchema {
            value_type,
            repeated: false,
            ordered: false,
        }
    }

    pub fn new_ordered(value_type: ValueType) -> Self {
        FieldMemberSchema {
            value_type,
            repeated: false,
            ordered: true,
        }
    }

    pub fn new_repeated(value_type: ValueType) -> Self {
        FieldMemberSchema {
            value_type,
            repeated: true,
            ordered: false,
        }
    }
}

impl From<FieldMemberSchema> for MemberSchema {
    fn from(field: FieldMemberSchema) -> Self {
        MemberSchema::Field(field)
    }
}

impl From<ResourceMemberSchema> for MemberSchema {
    fn from(resource: ResourceMemberSchema) -> Self {
        MemberSchema::Resource(resource)
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
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
        assert!(schema.is_ordered("user.id"));
        assert!(schema.is_repeated("task.tags"));
    }
}
