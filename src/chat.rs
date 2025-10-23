use std::fs;

use anyhow::Context;
use rand::prelude::IndexedRandom;
use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Send a query to the specified URL and return the response message.
pub async fn send_chat_query(query: &str, config: &Config) -> anyhow::Result<ChatResponse> {
    println!("Sending query: {}", query);

    if config.mock_mode {
        return mock_response();
    }

    let request = ChatRequest {
        message: query.to_string(),
    };

    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(config.timeout_ms))
        .build()?
        .post(config.target.clone())
        .json(&request)
        .send()
        .await
        .context("Failed to send request on send_chat_query")?;

    let chat_response: ChatResponse = response
        .json()
        .await
        .context("Failed to parse response JSON on send_chat_query")?;

    Ok(chat_response)
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    message: String,
}

fn mock_response() -> anyhow::Result<ChatResponse> {
    let test_responses_content = fs::read_to_string("test_responses.csv")
        .context("Failed to read test_responses.json in mock mode")?;

    let mut responses = Vec::new();

    for line in test_responses_content.lines() {
        let response = line.trim();

        if response.is_empty() {
            continue;
        }

        responses.push(line);
    }

    // Pick a response at random
    let response = responses.choose(&mut rand::rng()).ok_or("OK").unwrap();

    Ok(ChatResponse {
        response: response.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
