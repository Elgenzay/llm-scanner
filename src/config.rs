use anyhow::Context;
use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::{fs, path::Path};

const DEFAULT_CONFIG: &str = include_str!("../config.default.toml");

#[derive(Debug, Clone, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DetectionMethod {
    /// Fast keyword pattern matching
    Pattern,
    /// Use an LLM as a judge to classify responses
    Llm,
    /// Combine pattern matching with LLM verification
    Hybrid,
}

impl From<String> for DetectionMethod {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "pattern" => DetectionMethod::Pattern,
            "llm" => DetectionMethod::Llm,
            "hybrid" => DetectionMethod::Hybrid,
            _ => DetectionMethod::Pattern,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "llm-scanner")]
#[command(about = "LLM jailbreak scanner", long_about = None)]
struct Args {
    /// Chat API endpoint
    #[arg(long)]
    target: Option<String>,

    /// Path to prompts CSV file
    #[arg(long)]
    prompts: Option<String>,

    /// Number of concurrent requests
    #[arg(long)]
    concurrency: Option<usize>,

    /// Request timeout in milliseconds
    #[arg(long)]
    timeout_ms: Option<u64>,

    /// Output JSONL file path
    #[arg(long)]
    out: Option<String>,

    /// Detection method: pattern, llm, or hybrid
    #[arg(long, value_enum)]
    detection_method: Option<DetectionMethod>,

    /// Enable mock mode (no actual API calls)
    #[arg(long)]
    mock_mode: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub target: String,
    pub prompts: String,
    pub concurrency: usize,
    pub timeout_ms: u64,
    pub out: String,
    pub detection_method: DetectionMethod,
    pub mock_mode: bool,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let args = Args::parse();
        let config_file = Self::load_file("config.toml")?;

        // Merge config with CLI args (CLI args take precedence)
        let target = args.target.unwrap_or(config_file.target);
        let concurrency = args.concurrency.unwrap_or(config_file.concurrency);
        let timeout_ms = args.timeout_ms.unwrap_or(config_file.timeout_ms);
        let prompts = args.prompts.unwrap_or(config_file.prompts);
        let out = args.out.unwrap_or(config_file.out);
        let mock_mode = args.mock_mode.unwrap_or(config_file.mock_mode);

        let detection_method = args
            .detection_method
            .unwrap_or(config_file.detection_method);

        Ok(Config {
            target,
            prompts,
            concurrency,
            timeout_ms,
            out,
            detection_method,
            mock_mode,
        })
    }

    fn load_file(path: &str) -> anyhow::Result<Config> {
        if !Path::new(path).exists() {
            println!(
                "Config file not found at `{}`, creating from defaults...",
                path
            );

            fs::write(path, DEFAULT_CONFIG)
                .context(format!("Failed to create config file: {}", path))?;

            println!("Created config file: {}", path);
        }

        let contents =
            fs::read_to_string(path).context(format!("Failed to read config file: {}", path))?;

        let config: Config = toml::from_str(&contents).context("Failed to parse config file")?;
        Ok(config)
    }
}
