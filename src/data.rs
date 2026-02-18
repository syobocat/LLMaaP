use std::{os::unix::net::UnixListener, path::Path};

use anyhow::Context;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use serde::{Deserialize, Serialize};

use crate::{config::Config, llm::Message};

const SOCKET_PATH: &str = "/tmp/llmaap.sock";

#[derive(Serialize, Deserialize)]
struct SaveData {
    bootcount: u32,
    memory: String,
    context: Vec<Vec<Message>>,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            bootcount: 1,
            memory: String::new(),
            context: Vec::new(),
        }
    }
}

pub struct Data {
    pub bootcount: u32,
    pub config: Config,
    pub socket: UnixListener,
    pub memory: String,
    pub context: AllocRingBuffer<Vec<Message>>,
}

impl SaveData {
    fn load(path: &str) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(mut data) = serde_json::from_str::<Self>(&text) else {
            return Self::default();
        };
        data.bootcount += 1;
        data
    }

    fn save(&self, path: &str) -> anyhow::Result<()> {
        let text = serde_json::to_string(self).unwrap();
        std::fs::write(path, text).context("Failed to write the data")?;
        Ok(())
    }
}

impl Data {
    pub fn load(config: Config) -> Self {
        let savedata = SaveData::load(
            config
                .data_path
                .as_ref()
                .unwrap_or(&String::from("data.json")),
        );
        let mut context = AllocRingBuffer::new(config.context_size);
        for shard in savedata.context {
            context.enqueue(shard);
        }

        let socket_path = Path::new(SOCKET_PATH);
        if socket_path.exists() {
            std::fs::remove_file(socket_path).unwrap();
        }
        let socket = UnixListener::bind("/tmp/llmaap.sock").unwrap();

        Self {
            bootcount: savedata.bootcount,
            config,
            socket,
            memory: savedata.memory,
            context,
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let savedata = SaveData {
            bootcount: self.bootcount,
            memory: self.memory.clone(),
            context: self.context.to_vec(),
        };
        savedata.save(
            self.config
                .data_path
                .as_ref()
                .unwrap_or(&String::from("data.json")),
        )
    }
}
