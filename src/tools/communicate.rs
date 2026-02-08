use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixListener,
};

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
pub struct NotifyOutput {
    success: bool,
    result: String,
}

#[derive(Serialize)]
pub struct AskOutput {
    success: bool,
    result: String,
    response: Option<String>,
}

#[derive(Deserialize)]
pub struct Notify {
    message: String,
}

#[derive(Deserialize)]
pub struct Ask {
    query: String,
}

impl Notify {
    pub fn run(&self, webhook_url: &str) -> NotifyOutput {
        if let Err(e) = send_discord(webhook_url, &self.message) {
            NotifyOutput {
                success: false,
                result: format!("Failed to send the message to the admin: {e}"),
            }
        } else {
            NotifyOutput {
                success: true,
                result: String::from("Message sent."),
            }
        }
    }
}

impl Ask {
    pub fn run(&self, webhook_url: &str, socket: &UnixListener) -> AskOutput {
        if let Err(e) = send_discord(webhook_url, &self.query) {
            return AskOutput {
                success: false,
                result: format!("Failed to send the message to the admin: {e}"),
                response: None,
            };
        }
        match readline(socket) {
            Ok(response) => AskOutput {
                success: true,
                result: String::from("Message sent, reply received."),
                response: Some(response),
            },
            Err(e) => AskOutput {
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

    if stream.write_all(b"?> ").is_err() {
        return Err(String::from("Failed to keep the connection to the admin"));
    }

    let mut reader = BufReader::new(stream);

    let mut buf = String::new();
    if reader.read_line(&mut buf).is_err() {
        return Err(String::from("Failed to decode the response from the admin"));
    }

    Ok(buf.trim().to_owned())
}
