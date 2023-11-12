use crate::tools::serving_status_serde;
use prost::Name;
use serde::{Deserialize, Serialize};

use crate::tools::{
    command::command::Kind as CommandKind,
    command::{
        command::{status::Kind as StatusKind, Status},
        Command,
    },
    CommandRequest, ServingStatus,
};

#[allow(clippy::module_inception)]
pub mod tools {
    pub mod command {
        include!("./proto/tools.command.rs");
        include!("./proto/tools.command.plus.rs");
    }
    pub mod perms {
        include!("./proto/tools.perms.rs");
        include!("./proto/tools.perms.plus.rs");
    }
    include!("./proto/tools.rs");
    include!("./proto/tools.plus.rs");
}

#[test]
fn names() {
    assert_eq!(CommandRequest::NAME, "CommandRequest");
    assert_eq!(CommandRequest::PACKAGE, "tools");

    assert_eq!(Command::NAME, "Command");
    assert_eq!(Command::KIND_ONEOF_NAME, "kind");
    assert_eq!(CommandKind::STATUS_VARIANT_NAME, "status");

    assert_eq!(Status::NAME, "Command.Status");
    assert_eq!(Status::PACKAGE, "tools.command");
}

#[test]
fn converts() {
    assert!(matches!(
        CommandKind::from(Status {
            ..Default::default()
        }),
        CommandKind::Status(_)
    ));
    assert!(matches!(
        Status::from("error".to_string()),
        Status { kind: Some(StatusKind::Error(err)), .. } if err == "error"
    ));
    assert_eq!(
        Status::from("err".to_string())
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
        #[serde(with = "serving_status_serde")]
        status: i32,
    }

    assert_eq!(
        serde_json::from_str::<ServingStatus>(
            &serde_json::to_string(&ServingStatus::Serving).unwrap()
        )
        .unwrap(),
        ServingStatus::Serving,
    );

    let serialized = serde_json::to_string(&StatusObj {
        status: ServingStatus::Serving as i32,
    })
    .unwrap();
    assert_eq!(serialized, r#"{"status":"SERVING"}"#);
    assert_eq!(
        serde_json::from_str::<StatusObj>(&serialized)
            .unwrap()
            .status,
        ServingStatus::Serving as i32,
    );
}
