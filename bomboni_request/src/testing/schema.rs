use bomboni_macros::btree_map_into;

use crate::schema::{
    FieldMemberSchema, MemberSchema, ResourceMemberSchema, Schema, SchemaMapped, ValueType,
};
use crate::value::Value;

/// A test request item containing user and task data.
pub struct RequestItem {
    /// The user item.
    pub user: UserItem,
    /// The task item.
    pub task: TaskItem,
}

/// A test user item.
pub struct UserItem {
    /// The user ID.
    pub id: String,
    /// The user's display name.
    pub display_name: String,
    /// The user's age.
    pub age: i32,
}

/// A test task item.
pub struct TaskItem {
    /// The task ID.
    pub id: String,
    /// The user ID this task belongs to.
    pub user_id: String,
    /// The task content.
    pub content: String,
    /// Whether the task is deleted.
    pub deleted: bool,
    /// The task tags.
    pub tags: Vec<String>,
}

impl RequestItem {
    /// Gets the schema for the request item.
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
    /// Gets the schema for the user item.
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
    /// Gets the schema for the task item.
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
