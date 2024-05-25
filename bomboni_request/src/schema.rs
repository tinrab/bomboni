use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Schema {
    pub members: BTreeMap<String, MemberSchema>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldMemberSchema {
    pub value_type: ValueType,
    pub repeated: bool,
    pub ordered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSchema {
    pub argument_value_types: Vec<ValueType>,
    pub return_value_type: ValueType,
}

pub type FunctionSchemaMap = BTreeMap<String, FunctionSchema>;

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

    pub fn get_field(&self, name: &str) -> Option<&FieldMemberSchema> {
        if let Some(MemberSchema::Field(field)) = self.get_member(name) {
            Some(field)
        } else {
            None
        }
    }
}

impl FieldMemberSchema {
    pub fn new(value_type: ValueType) -> Self {
        Self {
            value_type,
            repeated: false,
            ordered: false,
        }
    }

    pub fn new_ordered(value_type: ValueType) -> Self {
        Self {
            value_type,
            repeated: false,
            ordered: true,
        }
    }

    pub fn new_repeated(value_type: ValueType) -> Self {
        Self {
            value_type,
            repeated: true,
            ordered: false,
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

#[cfg(feature = "testing")]
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
