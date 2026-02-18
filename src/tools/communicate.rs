use std::{
    io::{Read, Write},
    os::unix::net::UnixListener,
};

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
pub struct Output {
    success: bool,
    result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    response: Option<String>,
}

#[derive(Deserialize)]
pub struct Communicate {
    message: String,
    need_reply: bool,
}

impl Communicate {
    pub fn run(&self, webhook_url: &str, socket: &UnixListener) -> Output {
        if let Err(e) = send_discord(webhook_url, &self.message) {
            return Output {
                success: false,
                result: format!("Failed to send the message to the admin: {e}"),
                response: None,
            };
        }
        if !self.need_reply {
            return Output {
                success: true,
                result: String::from("Message sent."),
                response: None,
            };
        }
        match readline(socket) {
            Ok(response) => Output {
                success: true,
                result: String::from("Message sent, reply received."),
                response: Some(response),
            },
            Err(e) => Output {
                success: false,
                result: format!(
                    "The message was sent, but failed to receive a reply from the admin: {e}"
                ),
                response: None,
            },
        }
    }
}

fn send_discord(webhook_url: &str, message: &str) -> Result<(), String> {
    ureq::post(webhook_url)
        .send_json(json!({
            "content": message,
        }))
        .map_err(|e| format!("{e}"))?;
    Ok(())
}

fn readline(socket: &UnixListener) -> Result<String, String> {
    let Ok((mut stream, _)) = socket.accept() else {
        return Err(String::from("Failed to accept a connection from the admin"));
    };

    if stream
        .write_all(b"=== Waiting for a message ===\n")
        .is_err()
    {
        return Err(String::from("Failed to keep the connection to the admin"));
    }

    let mut buf = String::new();
    if stream.read_to_string(&mut buf).is_err() {
        return Err(String::from("Failed to decode the response from the admin"));
    }

    Ok(buf.trim().to_owned())
}
