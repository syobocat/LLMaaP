use serde_json::json;

use crate::data::Data;

mod communicate;
mod exec;
mod memory;

pub fn call_tool(name: &str, args: Option<&String>, data: &mut Data) -> String {
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
        "append_memory" => {
            let Some(args) = args else {
                return json!({
                    "err": "Failed to call `append_memory`: No arguments specified",
                })
                .to_string();
            };
            let Ok(append) = serde_json::from_str::<memory::AppendMemory>(args) else {
                return json!({
                    "err": "Failed to call `append_memory`: Malformed arguments",
                })
                .to_string();
            };
            append.run(&mut data.memory);

            json!({
                "msg": "Memory updated."
            })
            .to_string()
        }
        "replace_memory" => {
            let Some(args) = args else {
                return json!({
                    "err": "Failed to call `replace_memory`: No arguments specified",
                })
                .to_string();
            };
            let Ok(replace) = serde_json::from_str::<memory::ReplaceMemory>(args) else {
                return json!({
                    "err": "Failed to call `replace_memory`: Malformed arguments",
                })
                .to_string();
            };
            replace.run(&mut data.memory);

            json!({
                "msg": "Memory updated."
            })
            .to_string()
        }
        "get_memory" => json!({
            "content": data.memory
        })
        .to_string(),
        "shutdown" => {
            data.shutdown = true;
            json!({
                "msg": "Shutdown scheduled."
            })
            .to_string()
        }
        _ => json!({
            "err": format!("Failed to call a tool: Unknown function `{name}`")
        })
        .to_string(),
    }
}
