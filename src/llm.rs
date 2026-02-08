use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ureq::Agent;

use crate::config::Config;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum Message {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        content: String,
        tool_calls: Option<Vec<ToolCall>>,
    },
    Tool {
        content: String,
        tool_call_id: String,
    },
}

impl Message {
    pub fn from_resp_message(resp_message: ResponseMessage) -> Self {
        Self::Assistant {
            content: resp_message.content,
            tool_calls: resp_message.tool_calls,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    call_type: ToolCallType,
    pub function: Function,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallType {
    Function,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub arguments: Option<String>,
}

#[derive(Deserialize)]
pub struct Response {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub struct Choice {
    pub finish_reason: FinishReason,
    pub message: ResponseMessage,
}

#[derive(PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    ToolCalls,
}

#[derive(Clone, Deserialize)]
pub struct ResponseMessage {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

pub struct Client {
    agent: Agent,
}

impl Client {
    pub fn new() -> Self {
        Self {
            agent: ureq::agent(),
        }
    }
    pub fn send(&self, config: &Config, memory: Vec<Message>) -> anyhow::Result<Response> {
        let mut messages = vec![Message::System {
            content: config.system_prompt.clone(),
        }];
        if !matches!(memory[0], Message::User { content: _ }) {
            messages.push(Message::User {
                content: String::from("System: これ以前のログは切り捨てられています。"),
            });
        }
        messages.extend(memory);
        let req = json!({
            "model": config.model,
            "messages": messages,
            "tools": [
                {
                    "type": "function",
                    "function": {
                        "name": "exec",
                        "description": "Executes a UNIX command or a Shellscript",
                        "parameters": {
                            "type": "object",
                            "required": ["command"],
                            "additionalProperties": false,
                            "properties": {
                                "command": {
                                    "type": "string",
                                },
                            },
                        },
                    },
                },
                {
                    "type": "function",
                    "function": {
                        "name": "ask",
                        "description": "Ask the admin",
                        "parameters": {
                            "type": "object",
                            "required": ["query"],
                            "additionalProperties": false,
                            "properties": {
                                "query": {
                                    "type": "string",
                                },
                            },
                        },
                    },
                },
                {
                    "type": "function",
                    "function": {
                        "name": "notify",
                        "description": "Notify the admin",
                        "parameters": {
                            "type": "object",
                            "required": ["message"],
                            "additionalProperties": false,
                            "properties": {
                                "message": {
                                    "type": "string",
                                },
                            },
                        },
                    },
                },
                {
                    "type": "function",
                    "function": {
                        "name": "shutdown",
                        "description": "Schedule a shutdown; Use only when requested by the admin",
                    },
                },
            ],
        });

        let mut request = self
            .agent
            .post(format!("{}/chat/completions", config.endpoint));

        if let Some(ref token) = config.token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        let response: Response = request
            .send_json(req)
            .context("Failed to send a request")?
            .body_mut()
            .read_json()
            .context("Failed to parse json")?;

        Ok(response)
    }
}
