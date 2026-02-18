use std::{fmt::Write, time::Duration};

use chrono::Local;
use env_logger::Env;
use ringbuffer::RingBuffer;

use crate::{
    config::Config,
    data::Data,
    llm::{FinishReason, Message},
};

mod config;
mod data;
mod llm;
mod tools;

fn load_config(
    path_override: Option<String>,
    endpoint_override: Option<String>,
    token_override: Option<String>,
    model_override: Option<String>,
    data_path_override: Option<String>,
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
    if let Some(data_path) = data_path_override {
        config.data_path = Some(data_path);
    }
    config
}

pub fn boot(
    config_path_override: Option<String>,
    data_path_override: Option<String>,
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
        data_path_override,
    );
    let mut data = Data::load(config);

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
    data.context.enqueue(vec![Message::User {
        content: boot_message,
    }]);

    while !data.shutdown {
        loop {
            let Ok(resp) = client.send(
                &data.config,
                data.context.clone().into_iter().flatten().collect(),
            ) else {
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
                    let result = tools::call_tool(name, function.arguments.as_ref(), &mut data);
                    message_buffer.push(Message::Tool {
                        content: result,
                        tool_call_id: call.id.clone(),
                    });
                }
            }
            data.context.enqueue(message_buffer);
            data.save().expect("Savefile should be writebale");

            if choice.finish_reason == FinishReason::Stop {
                log::info!("Sending heartbeat...");
                let heartbeat_message = format!(
                    "System: heartbeat; 現在時刻: {}",
                    Local::now().format("%Y年%m月%d日 %H時%M分"),
                );
                data.context.enqueue(vec![Message::User {
                    content: heartbeat_message,
                }]);
                break;
            }
        }
    }
}
