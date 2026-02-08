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

pub fn call_notify(args: Option<&String>, webhook_url: &str) -> String {
    let Some(args) = args else {
        return json!({
            "err": "Failed to call `notify`: No arguments specified",
        })
        .to_string();
    };
    let Ok(notify) = serde_json::from_str::<communicate::Notify>(args) else {
        return json!({
            "err": "Failed to call `notify`: Malformed arguments",
        })
        .to_string();
    };
    serde_json::to_string(&notify.run(webhook_url)).unwrap()
}

pub fn call_ask(args: Option<&String>, webhook_url: &str, socket: &UnixListener) -> String {
    let Some(args) = args else {
        return json!({
            "err": "Failed to call `ask`: No arguments specified",
        })
        .to_string();
    };
    let Ok(ask) = serde_json::from_str::<communicate::Ask>(args) else {
        return json!({
            "err": "Failed to call `ask`: Malformed arguments",
        })
        .to_string();
    };
    serde_json::to_string(&ask.run(webhook_url, socket)).unwrap()
}
