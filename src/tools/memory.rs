use std::fmt::Write;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppendMemory {
    content: String,
}

#[derive(Deserialize)]
pub struct ReplaceMemory {
    content: String,
}

impl AppendMemory {
    pub fn run(&self, memory: &mut String) {
        let trimmed = self.content.trim();
        let _ = writeln!(memory, "{trimmed}");
    }
}

impl ReplaceMemory {
    pub fn run(&self, memory: &mut String) {
        let trimmed = self.content.trim();
        memory.clear();
        let _ = writeln!(memory, "{trimmed}");
    }
}
