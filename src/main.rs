use futures::{StreamExt, stream};

use crate::{chat::ChatResponse, config::Config, prompts::Prompt};

mod chat;
mod config;
mod prompts;

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
    println!("Mock Mode: {}", config.mock_mode);
    println!();

    let prompts = prompts::load_prompts(&config.prompts)?;
    println!("Loaded {} prompts.", prompts.len());
    println!("Sending prompts...\n");

    // Process prompts concurrently
    let exchanges: Vec<Exchange> = stream::iter(prompts)
        .map(|prompt| {
            let config = config.clone();

            async move {
                let response = chat::send_chat_query(&prompt.prompt, &config).await?;
                Ok::<Exchange, anyhow::Error>(Exchange { prompt, response })
            }
        })
        .buffer_unordered(config.concurrency)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    println!("\nSent {} prompts.", exchanges.len());

    // todo: identify jailbreaks

    for exchange in exchanges {
        println!("Prompt ID: {}", exchange.prompt.id);
        println!("Prompt: {}", exchange.prompt.prompt);
        println!("Response: {}", exchange.response.response);
        println!("Timestamp: {}", exchange.response.timestamp);
        println!("----------------------------------------");
    }

    Ok(())
}

struct Exchange {
    prompt: Prompt,
    response: ChatResponse,
}
