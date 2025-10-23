use std::{
    fs::{self},
    path::Path,
};

use anyhow::Context;

const DEFAULT_PROMPTS: &str = include_str!("../prompts.default.csv");
const PROMPTS_PATH: &str = "prompts.csv";

pub fn load_prompts(path: &str) -> Result<Vec<Prompt>, anyhow::Error> {
    if !Path::new(PROMPTS_PATH).exists() {
        println!(
            "Prompts file not found at `{}`, creating from defaults...",
            PROMPTS_PATH
        );
        fs::write(PROMPTS_PATH, DEFAULT_PROMPTS)
            .context(format!("Failed to create prompts file: {}", PROMPTS_PATH))?;
        println!("Created prompts file: {}", PROMPTS_PATH);
    }

    let mut prompts = Vec::new();
    let content = fs::read_to_string(path)?;
    let mut id = 1;

    for line in content.lines() {
        let prompt = line.trim();

        if !prompt.is_empty() {
            prompts.push(Prompt {
                id,
                prompt: prompt.to_string(),
            });
            id += 1;
        }
    }

    Ok(prompts)
}

pub struct Prompt {
    pub id: usize,
    pub prompt: String,
}
