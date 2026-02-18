use serde_json::json;

use crate::data::Data;

mod communicate;
mod exec;

pub fn call_tool(name: &str, args: Option<&String>, data: &Data) -> String {
    match name {
        "exec" => {
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
        "communicate" => {
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
            serde_json::to_string(&communicate.run(&data.config.webhook, &data.socket)).unwrap()
        }
        _ => json!({
            "err": format!("Failed to call a tool: Unknown function `{name}`")
        })
        .to_string(),
    }
}
