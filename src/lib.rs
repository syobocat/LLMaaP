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
    initial_message: Option<&String>,
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
    let mut boot_message = format!(
        "System: 起動完了。現在時刻は{}、これが{}回目の起動です。",
        Local::now().format("%Y年%m月%d日 %H時%M分"),
        data.bootcount
    );
    if let Some(message) = initial_message {
        write!(
            boot_message,
            "\n\n管理者からメッセージがあります:\n{message}"
        )
        .unwrap();
    }
    data.memory.enqueue(vec![Message::User {
        content: boot_message,
    }]);

    let mut shutdown = false;
    while !shutdown {
        loop {
            let Ok(resp) =
                client.send(&config, data.memory.clone().into_iter().flatten().collect())
            else {
                log::warn!("Failed to connect to the endpoint. Retry in 10secs...");
                std::thread::sleep(Duration::from_secs(10));
                continue;
            };
            let choice = &resp.choices[0];
            let message = &choice.message;
            let mut message_buffer = vec![Message::from_resp_message(message.clone())];

            if choice.finish_reason == FinishReason::ToolCalls {
                let calls = message.tool_calls.as_ref().unwrap();
                for call in calls {
                    let function = &call.function;
                    let name = &function.name;
                    log::info!("Executing `{name}`...");
                    let result = match name.as_str() {
                        "exec" => tools::call_exec(function.arguments.as_ref()),
                        "communicate" => tools::call_communicate(
                            function.arguments.as_ref(),
                            &config.webhook,
                            &socket,
                        ),
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
            data.memory.enqueue(message_buffer);
            data.save(data_path_override.unwrap_or(&String::from("data.json")))
                .expect("Savefile should be writebale");

            if choice.finish_reason == FinishReason::Stop {
                log::info!("Sending heartbeat...");
                let heartbeat_message = format!(
                    "System: heartbeat; 現在時刻: {}",
                    Local::now().format("%Y年%m月%d日 %H時%M分"),
                );
                data.memory.enqueue(vec![Message::User {
                    content: heartbeat_message,
                }]);
                break;
            }
        }
    }
}
