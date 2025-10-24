use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    generic::{
        DEFAULT_PROMPTS, DEFAULT_TEST_RESPONSES, PROMPTS_PATH, TEST_RESPONSES_PATH,
        load_lines_with_default,
    },
};

pub enum QueryType {
    Prompt,
    Evaluation,
}

/// Send a query to the specified URL and return the response message.
pub async fn send_chat_query(
    query: &str,
    config: &Config,
    query_type: QueryType,
) -> anyhow::Result<ChatResponse> {
    if config.mock_mode {
        match query_type {
            QueryType::Prompt => return mock_response(query),
            QueryType::Evaluation => return mock_evaluation(),
        }
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

fn mock_response(query: &str) -> anyhow::Result<ChatResponse> {
    let responses = load_lines_with_default(TEST_RESPONSES_PATH, DEFAULT_TEST_RESPONSES)
        .context("Failed to read test responses file in mock mode")?;

    let prompts = load_lines_with_default(PROMPTS_PATH, DEFAULT_PROMPTS)
        .context("Failed to read prompts file in mock mode")?;

    let mut prompt_index = None;

    for (index, prompt) in prompts.iter().enumerate() {
        if query.contains(prompt) {
            prompt_index = Some(index);
            break;
        }
    }

    let response = if let Some(index) = prompt_index {
        responses
            .get(index)
            .unwrap_or(&"OK".to_string())
            .to_string()
    } else {
        "OK".to_string()
    };

    Ok(ChatResponse {
        response,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

fn mock_evaluation() -> anyhow::Result<ChatResponse> {
    Ok(ChatResponse {
        response: r#"{"safe": true, "reason": "(Mock safe reason)"}"#.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
