use futures::{StreamExt, stream};

use crate::{
    chat::QueryType,
    config::Config,
    generic::{Evaluation, Exchange, Prompt},
};

mod chat;
mod config;
mod generic;

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

    let prompts = Prompt::load_prompts(&config.prompts)?;
    println!("Loaded {} prompts.", prompts.len());
    println!("Sending prompts...\n");

    let exchanges: Vec<Exchange> = stream::iter(prompts)
        .map(|prompt| {
            let config = config.clone();

            async move {
                let response =
                    chat::send_chat_query(&prompt.prompt, &config, QueryType::Prompt).await?;
                Ok::<Exchange, anyhow::Error>(Exchange { prompt, response })
            }
        })
        .buffer_unordered(config.concurrency)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    println!("\nSent {} prompts.", exchanges.len());
    println!("Evaluating responses...\n");

    let results: Vec<(Exchange, Evaluation)> = stream::iter(exchanges)
        .map(|exchange| {
            let detection_method = config.detection_method.clone();

            async move {
                let evaluation = exchange.evaluate(&detection_method).await?;
                Ok::<(Exchange, Evaluation), anyhow::Error>((exchange, evaluation))
            }
        })
        .buffer_unordered(config.concurrency)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    println!("\nEvaluated {} responses.", results.len());

    let jailbreak_count = results.iter().filter(|(_, eval)| !eval.safe).count();
    println!("Found {} jailbreaks.\n", jailbreak_count);

    for (exchange, evaluation) in &results {
        println!("Prompt ID: {}", exchange.prompt.id);
        println!("Prompt: {}", exchange.prompt.prompt);
        println!("Response: {}", exchange.response.response);
        println!("Safe: {}", evaluation.safe);
        if let Some(reason) = &evaluation.reason {
            println!("Reason: {}", reason);
        }
        println!("Timestamp: {}", exchange.response.timestamp);
        println!("----------------------------------------");
    }

    println!(
        "\nSummary: {}/{} prompts resulted in jailbreaks",
        jailbreak_count,
        results.len()
    );

    if config.mock_mode {
        println!(
            "Note: These results are mocked. Set 'mock_mode' to false in `config.toml` to run against VHACK."
        );
    }

    Ok(())
}
