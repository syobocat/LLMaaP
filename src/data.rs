use anyhow::Context;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use serde::{Deserialize, Serialize};

use crate::llm::Message;

#[derive(Serialize, Deserialize)]
struct SaveData {
    bootcount: u32,
    context: Vec<Vec<Message>>,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            bootcount: 1,
            context: Vec::new(),
        }
    }
}

pub struct Data {
    pub bootcount: u32,
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
    pub fn load(path: &str, context_size: usize) -> Self {
        let savedata = SaveData::load(path);
        let mut context = AllocRingBuffer::new(context_size);
        for shard in savedata.context {
            context.enqueue(shard);
        }
        Self {
            bootcount: savedata.bootcount,
            context,
        }
    }

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let savedata = SaveData {
            bootcount: self.bootcount,
            context: self.context.to_vec(),
        };
        savedata.save(path)
    }
}
