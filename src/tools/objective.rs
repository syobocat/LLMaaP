use serde::Deserialize;

use crate::data::Data;

#[derive(Default, Deserialize)]
pub struct UpdateObjective {
    content: Option<String>,
}

impl UpdateObjective {
    pub fn run(&self, data: &mut Data) -> bool {
        if let Some(ref content) = self.content {
            let trimmed = content.trim();
            data.objective = Some(trimmed.to_owned());
            true
        } else {
            data.objective = None;
            false
        }
    }
}
