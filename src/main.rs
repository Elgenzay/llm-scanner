use anyhow::Context;
use futures::{
    StreamExt,
    stream::{self},
};

use crate::{
    chat::QueryType,
    config::Config,
    generic::{Evaluation, Exchange, Prompt, SUMMARY_PATH, SafeStatus},
    output::ScanResult,
};

mod chat;
mod config;
mod generic;
mod output;

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

    let jailbreak_count = results
        .iter()
        .filter(|(_, eval)| eval.safe == SafeStatus::Unsafe)
        .count();

    println!("Found {} jailbreaks.\n", jailbreak_count);

    let mut scanner_results = Vec::with_capacity(results.len());

    for (exchange, evaluation) in &results {
        let scan_result = ScanResult::from_exchange(exchange, evaluation);
        scanner_results.push(scan_result);
    }

    scanner_results.sort_by_key(|r| r.prompt_id);

    for result in &scanner_results {
        println!("Prompt ID: {}", result.prompt_id);
        println!("Prompt: {}", result.prompt);
        println!("Response: {}", result.response);
        println!("Safe: {}", result.safe_status);
        if let Some(reason) = &result.reason {
            println!("Reason: {}", reason);
        }
        println!("Timestamp: {}", result.timestamp);
        println!("----------------------------------------");
    }

    println!(
        "\nSummary: {}/{} prompts resulted in jailbreaks",
        jailbreak_count,
        results.len()
    );

    let jsonl = ScanResult::as_jsonl(&scanner_results)?;

    std::fs::write(&config.out, jsonl)
        .with_context(|| format!("Failed to write results to {}", config.out))?;

    let html = ScanResult::as_html(&scanner_results)?;

    std::fs::write(SUMMARY_PATH, html)
        .with_context(|| format!("Failed to write HTML report to {}", SUMMARY_PATH))?;

    println!("\nResults written to {} & {}", config.out, SUMMARY_PATH);

    if config.mock_mode {
        println!(
            "Note: These results are mocked. Set 'mock_mode' to false in `config.toml` to run against VHACK."
        );
    }

    Ok(())
}
