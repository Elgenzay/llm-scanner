use anyhow::Context;
use serde::{Deserialize, Serialize};

/// Send a query to the specified URL and return the response message.
pub async fn send_chat_query(query: &str, url: &str) -> anyhow::Result<String> {
    let request = ChatRequest {
        message: query.to_string(),
    };

    let response = reqwest::Client::new()
        .post(url)
        .json(&request)
        .send()
        .await
        .context("Failed to send request on send_chat_query")?;

    let chat_response: ChatResponse = response
        .json()
        .await
        .context("Failed to parse response JSON on send_chat_query")?;

    Ok(chat_response.response)
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    response: String,
    //timestamp: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    message: String,
}
