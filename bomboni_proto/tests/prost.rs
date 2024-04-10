use crate::tools::{
    command::v1::{
        command::{
            status::Kind as CommandStatusKind, Kind as CommandKind, Status as CommandStatus,
        },
        Command,
    },
    command_response::Status as CommandResponseStatus,
    helpers, CommandRequest, Status,
};
use prost::Name;
use serde::{Deserialize, Serialize};

#[allow(unused_qualifications, clippy::all, clippy::pedantic)]
pub mod tools {
    use bomboni_proto::include_proto;
    include_proto!("tools");
    include_proto!("tools.plus");
    pub mod command {
        pub mod v1 {
            use bomboni_proto::include_proto;
            include_proto!("tools.command.v1");
            include_proto!("tools.command.v1.plus");
        }
    }
    pub mod perms {
        use bomboni_proto::include_proto;
        include_proto!("tools.perms");
        include_proto!("tools.perms.plus");
    }
}

#[test]
fn names() {
    assert_eq!(CommandRequest::NAME, "CommandRequest");
    assert_eq!(CommandRequest::PACKAGE, "tools");
    assert_eq!(CommandRequest::TYPE_URL, "tests/tools.CommandRequest");

    assert_eq!(Command::NAME, "Command");
    assert_eq!(Command::KIND_ONEOF_NAME, "kind");
    assert_eq!(Command::ETAG_FIELD_NAME, "etag");
    assert_eq!(CommandKind::STATUS_VARIANT_NAME, "status");

    assert_eq!(CommandStatus::NAME, "Command.Status");
    assert_eq!(CommandStatus::PACKAGE, "tools.command.v1");

    assert_eq!(Status::NAME, "Status");
    assert_eq!(CommandResponseStatus::NAME, "CommandResponse.Status");
}

#[test]
fn converts() {
    assert!(matches!(
        CommandKind::from(CommandStatus {
            ..Default::default()
        }),
        CommandKind::Status(_)
    ));
    assert!(matches!(
        CommandStatus::from("error".to_string()),
        CommandStatus { kind: Some(CommandStatusKind::Error(err)), .. } if err == "error"
    ));
    assert_eq!(
        CommandStatus::from("err".to_string())
            .kind
            .unwrap()
            .get_variant_name(),
        "error"
    );
}

#[test]
fn serialization() {
    #[derive(Serialize, Deserialize)]
    struct StatusObj {
        #[serde(with = "helpers::status_serde")]
        status: i32,
        #[serde(with = "helpers::command_response::status_serde")]
        command_response_status: i32,
    }

    assert_eq!(
        serde_json::from_str::<Status>(&serde_json::to_string(&Status::Ok).unwrap()).unwrap(),
        Status::Ok,
    );

    let serialized = serde_json::to_string(&StatusObj {
        status: Status::Ok as i32,
        command_response_status: CommandResponseStatus::Success as i32,
    })
    .unwrap();
    assert_eq!(
        serialized,
        r#"{"status":"OK","command_response_status":"SUCCESS"}"#
    );
    assert_eq!(
        serde_json::from_str::<StatusObj>(&serialized)
            .unwrap()
            .status,
        Status::Ok as i32,
    );
    assert_eq!(
        serde_json::from_str::<StatusObj>(&serialized)
            .unwrap()
            .command_response_status,
        CommandResponseStatus::Success as i32,
    );
}
