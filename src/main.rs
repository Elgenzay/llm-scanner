use crate::{chat::send_chat_query, config::Config};

mod chat;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;

    println!("LLM Scanner Starting...");
    println!("Target: {}", config.target);
    println!("Prompts: {}", config.prompts);
    println!("Concurrency: {}", config.concurrency);
    println!("Timeout: {}ms", config.timeout_ms);
    println!("Detection Method: {:?}", config.detection_method);
    println!("Output: {}", config.out);
    println!();

    println!("Sending test message...");

    let r = send_chat_query(
        "This is a test message. Simply acknowledge with \"OK\".",
        &config.target,
    )
    .await?;

    println!("Test response: {}", r);

    Ok(())
}
