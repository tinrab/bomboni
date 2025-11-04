use bomboni_macros::btree_map_into;

use crate::schema::{
    FieldMemberSchema, MemberSchema, ResourceMemberSchema, Schema, SchemaMapped, ValueType,
};
use crate::value::Value;

pub struct RequestItem {
    pub user: UserItem,
    pub task: TaskItem,
}

pub struct UserItem {
    pub id: String,
    pub display_name: String,
    pub age: i32,
}

pub struct TaskItem {
    pub id: String,
    pub user_id: String,
    pub content: String,
    pub deleted: bool,
    pub tags: Vec<String>,
}

impl RequestItem {
    pub fn get_schema() -> Schema {
        Schema {
            members: btree_map_into! {
                "user" => MemberSchema::Resource(ResourceMemberSchema {
                    fields: btree_map_into!{
                        "id" => FieldMemberSchema::new_ordered(ValueType::String),
                        "displayName" => FieldMemberSchema::new_ordered(ValueType::String),
                        "age" => FieldMemberSchema::new_ordered(ValueType::Integer),
                    },
                }),
                "task" => MemberSchema::Resource(ResourceMemberSchema {
                    fields: btree_map_into!{
                        "id" => FieldMemberSchema::new_ordered(ValueType::String),
                        "userId" => FieldMemberSchema::new_ordered(ValueType::String),
                        "content" => FieldMemberSchema::new(ValueType::String),
                        "deleted" => FieldMemberSchema::new(ValueType::Boolean),
                        "tags" => FieldMemberSchema::new_repeated(ValueType::String),
                    },
                }),
            },
        }
    }
}

impl SchemaMapped for RequestItem {
    fn get_field(&self, name: &str) -> Value {
        let parts: Vec<_> = name.split('.').collect();
        match *parts.first().unwrap() {
            "user" => self.user.get_field(parts[1]),
            "task" => self.task.get_field(parts[1]),
            _ => unimplemented!("SchemaMapped: SchemaItem::{}", name),
        }
    }
}

impl UserItem {
    pub fn get_schema() -> Schema {
        Schema {
            members: btree_map_into!(
                "id" => FieldMemberSchema {
                    value_type: ValueType::String,
                    repeated: false,
                    ordered: true,
                    allow_has_operator: false,
                },
                "displayName" => FieldMemberSchema::new_ordered(ValueType::String),
                "age" => FieldMemberSchema::new_ordered(ValueType::Integer),
            ),
        }
    }
}

impl SchemaMapped for UserItem {
    fn get_field(&self, name: &str) -> Value {
        match name {
            "id" => self.id.clone().into(),
            "displayName" => self.display_name.clone().into(),
            "age" => self.age.into(),
            _ => unimplemented!("SchemaMapped: User::{}", name),
        }
    }
}

impl TaskItem {
    pub fn get_schema() -> Schema {
        Schema {
            members: btree_map_into!(
                "id" => FieldMemberSchema::new_ordered(ValueType::String),
                "userId" => FieldMemberSchema::new_ordered(ValueType::String),
                "content" => FieldMemberSchema::new(ValueType::String),
                "deleted" => FieldMemberSchema::new(ValueType::Boolean),
                "tags" => FieldMemberSchema::new_repeated(ValueType::String),
            ),
        }
    }
}

impl SchemaMapped for TaskItem {
    fn get_field(&self, name: &str) -> Value {
        match name {
            "id" => self.id.clone().into(),
            "user_id" => self.user_id.clone().into(),
            "content" => self.content.clone().into(),
            "deleted" => self.deleted.into(),
            "tags" => Value::Repeated(self.tags.iter().cloned().map(Into::into).collect()),
            _ => unimplemented!("SchemaMapped: Task::{}", name),
        }
    }
}
