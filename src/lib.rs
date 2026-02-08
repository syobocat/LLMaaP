use std::{fmt::Write, os::unix::net::UnixListener, path::Path, time::Duration};

use chrono::Local;
use env_logger::Env;
use ringbuffer::RingBuffer;
use serde_json::json;

use crate::{
    config::Config,
    data::Data,
    llm::{FinishReason, Message},
};

mod config;
mod data;
mod llm;
mod tools;

const SOCKET_PATH: &str = "/tmp/llmaap.sock";

fn load_config(
    path_override: Option<String>,
    endpoint_override: Option<String>,
    token_override: Option<String>,
    model_override: Option<String>,
) -> Config {
    let mut config = Config::load(&path_override.unwrap_or(String::from("config.toml")))
        .expect("Config file should be readable");
    if let Some(endpoint) = endpoint_override {
        config.endpoint = endpoint;
    }
    if let Some(token) = token_override {
        config.token = Some(token);
    }
    if let Some(model) = model_override {
        config.model = model;
    }
    config
}

pub fn boot(
    config_path_override: Option<String>,
    data_path_override: Option<&String>,
    endpoint_override: Option<String>,
    token_override: Option<String>,
    model_override: Option<String>,
    initial_message: Option<String>,
) {
    env_logger::init_from_env(Env::default().default_filter_or("llmaap=info"));

    let config = load_config(
        config_path_override,
        endpoint_override,
        token_override,
        model_override,
    );
    let mut data = Data::load(
        data_path_override.unwrap_or(&String::from("data.json")),
        config.context_size,
    );

    let socket_path = Path::new(SOCKET_PATH);
    if socket_path.exists() {
        std::fs::remove_file(socket_path).unwrap();
    }
    let socket = UnixListener::bind("/tmp/llmaap.sock").unwrap();

    let client = llm::Client::new();

    log::info!("Booting...");
    let mut boot = true;
    let mut shutdown = false;
    while !shutdown {
        let mut message_buffer = Vec::new();
        let now = Local::now();
        let system_message = if boot {
            let mut boot_message = format!(
                "System: 起動完了。現在時刻は{}、これが{}回目の起動です。",
                now.format("%Y年%m月%d日 %H時%M分"),
                data.bootcount
            );
            if let Some(ref message) = initial_message {
                write!(
                    boot_message,
                    "\n\n管理者からメッセージがあります:\n{message}"
                )
                .unwrap();
            }
            boot = false;
            boot_message
        } else {
            log::info!("Sending heartbeat...");
            format!(
                "System: heartbeat; 現在時刻: {}",
                now.format("%Y年%m月%d日 %H時%M分"),
            )
        };
        message_buffer.push(Message::User {
            content: system_message,
        });
        loop {
            let Ok(resp) = client.send(
                &config,
                data.memory.clone().into_iter().flatten().collect(),
                message_buffer.clone(),
            ) else {
                log::warn!("Failed to connect to the endpoint. Retry in 10secs...");
                std::thread::sleep(Duration::from_secs(10));
                continue;
            };
            let choice = &resp.choices[0];
            let message = &choice.message;
            message_buffer.push(Message::from_resp_message(message.clone()));

            if choice.finish_reason == FinishReason::ToolCalls {
                let calls = message.tool_calls.as_ref().unwrap();
                for call in calls {
                    let function = &call.function;
                    let name = &function.name;
                    log::info!("Executing `{name}`...");
                    let result = match name.as_str() {
                        "exec" => tools::call_exec(function.arguments.as_ref()),
                        "notify" => {
                            tools::call_notify(function.arguments.as_ref(), &config.webhook)
                        }
                        "ask" => {
                            tools::call_ask(function.arguments.as_ref(), &config.webhook, &socket)
                        }
                        "shutdown" => {
                            shutdown = true;
                            json!({
                                "msg": "Shutdown scheduled."
                            })
                            .to_string()
                        }
                        _ => json!({
                            "err": format!("Failed to call a tool: Unknown function `{name}`")
                        })
                        .to_string(),
                    };
                    message_buffer.push(Message::Tool {
                        content: result,
                        tool_call_id: call.id.clone(),
                    });
                }
            }

            message_buffer.push(Message::Assistant {
                content: String::new(),
                tool_calls: None,
            });
            data.memory.enqueue(message_buffer);
            data.save(data_path_override.unwrap_or(&String::from("data.json")))
                .expect("Savefile should be writebale");
            break;
        }
    }
}
