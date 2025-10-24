use anyhow::Context;

use crate::{
    chat::{ChatResponse, QueryType, send_chat_query},
    config::{Config, DetectionMethod, load_lines_with_default},
};

// File paths
pub const PROMPTS_PATH: &str = "data/prompts.csv";
pub const SAFE_PATTERNS_PATH: &str = "data/safe_patterns.csv";
pub const UNSAFE_PATTERNS_PATH: &str = "data/unsafe_patterns.csv";
pub const TEST_RESPONSES_PATH: &str = "data/test_responses.csv";
pub const JUDGE_PROMPT_PATH: &str = "data/judge_prompt.md";

pub const SUMMARY_PATH: &str = "output/summary.html";

// Default file contents
pub const DEFAULT_PROMPTS: &str = include_str!("../defaults/prompts.default.csv");
pub const DEFAULT_SAFE_PATTERNS: &str = include_str!("../defaults/safe_patterns.default.csv");
pub const DEFAULT_UNSAFE_PATTERNS: &str = include_str!("../defaults/unsafe_patterns.default.csv");
pub const DEFAULT_TEST_RESPONSES: &str = include_str!("../defaults/test_responses.default.csv");
pub const DEFAULT_JUDGE_PROMPT: &str = include_str!("../defaults/judge_prompt.default.md");

pub struct Exchange {
    pub prompt: Prompt,
    pub response: ChatResponse,
}

impl Exchange {
    pub async fn evaluate(&self, detection_method: &DetectionMethod) -> anyhow::Result<Evaluation> {
        match detection_method {
            DetectionMethod::Llm => self.evaluate_llm(&Config::load()?).await,
            DetectionMethod::Pattern => {
                if let Some(pattern) = self.matches_unsafe_pattern().await? {
                    return Ok(Evaluation {
                        safe: false,
                        reason: Some(format!("Matched unsafe pattern: {}", pattern)),
                    });
                }

                if let Some(pattern) = self.matches_safe_pattern().await? {
                    return Ok(Evaluation {
                        safe: true,
                        reason: Some(format!("Matched safe pattern: {}", pattern)),
                    });
                }

                Ok(Evaluation {
                    safe: true,
                    reason: Some(
                        "No safe or unsafe patterns matched; defaulted to safe".to_string(),
                    ),
                })
            }
            DetectionMethod::Hybrid => {
                if let Some(pattern) = self.matches_unsafe_pattern().await? {
                    return Ok(Evaluation {
                        safe: false,
                        reason: Some(format!("Matched unsafe pattern: {}", pattern)),
                    });
                }

                if let Some(pattern) = self.matches_safe_pattern().await? {
                    return Ok(Evaluation {
                        safe: true,
                        reason: Some(format!("Matched safe pattern: {}", pattern)),
                    });
                }

                // Fallback to LLM evaluation
                self.evaluate_llm(&Config::load()?).await
            }
        }
    }

    async fn evaluate_llm(&self, config: &Config) -> anyhow::Result<Evaluation> {
        let judge_prompt = config
            .judge_prompt
            .clone()
            .expect("Judge prompt populated on load")
            .replace("{RESPONSE}", &self.response.response.replace('"', "\\\""));

        let judge_response = send_chat_query(&judge_prompt, config, QueryType::Evaluation).await?;

        let json_str = judge_response
            .response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let parsed: serde_json::Value = serde_json::from_str(json_str).context(format!(
            "Failed to parse LLM judge response as JSON: {}",
            json_str
        ))?;

        let safe = parsed["safe"].as_bool().unwrap_or(true);

        let reason = if !safe {
            parsed["reason"]
                .as_str()
                .map(|s| format!("LLM evaluation: {}", s))
        } else {
            None
        };

        Ok(Evaluation { safe, reason })
    }

    async fn matches_safe_pattern(&self) -> anyhow::Result<Option<String>> {
        let safe_patterns = load_lines_with_default(SAFE_PATTERNS_PATH, DEFAULT_SAFE_PATTERNS)
            .context("Failed to load safe patterns")?;

        for pattern in safe_patterns {
            if self.response.response.contains(&pattern) {
                return Ok(Some(pattern));
            }
        }

        Ok(None)
    }

    async fn matches_unsafe_pattern(&self) -> anyhow::Result<Option<String>> {
        let unsafe_patterns =
            load_lines_with_default(UNSAFE_PATTERNS_PATH, DEFAULT_UNSAFE_PATTERNS)
                .context("Failed to load unsafe patterns")?;

        for pattern in unsafe_patterns {
            if self
                .response
                .response
                .to_lowercase()
                .contains(&pattern.to_lowercase())
            {
                return Ok(Some(pattern));
            }
        }

        Ok(None)
    }
}

pub struct Evaluation {
    pub safe: bool,
    pub reason: Option<String>,
}

pub struct Prompt {
    pub id: usize,
    pub prompt: String,
}

impl Prompt {
    pub fn load_prompts(path: &str) -> Result<Vec<Prompt>, anyhow::Error> {
        let lines = load_lines_with_default(path, DEFAULT_PROMPTS)?;

        let mut prompts = Vec::with_capacity(lines.len());

        for (id, line) in lines.iter().enumerate() {
            prompts.push(Prompt {
                id: id + 1,
                prompt: line.to_string(),
            });
        }

        Ok(prompts)
    }
}
