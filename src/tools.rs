use std::os::unix::net::UnixListener;

use serde_json::json;

mod communicate;
mod exec;

pub fn call_exec(args: Option<&String>) -> String {
    let Some(args) = args else {
        return json!({
            "err": "Failed to call `exec`: No arguments specified",
        })
        .to_string();
    };
    let Ok(exec) = serde_json::from_str::<exec::Exec>(args) else {
        return json!({
            "err": "Failed to call `exec`: Malformed arguments",
        })
        .to_string();
    };
    serde_json::to_string(&exec.run()).unwrap()
}

pub fn call_communicate(args: Option<&String>, webhook_url: &str, socket: &UnixListener) -> String {
    let Some(args) = args else {
        return json!({
            "err": "Failed to call `communicate`: No arguments specified",
        })
        .to_string();
    };
    let Ok(communicate) = serde_json::from_str::<communicate::Communicate>(args) else {
        return json!({
            "err": "Failed to call `communicate`: Malformed arguments",
        })
        .to_string();
    };
    serde_json::to_string(&communicate.run(webhook_url, socket)).unwrap()
}
