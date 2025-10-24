use serde::Serialize;

use crate::generic::{Evaluation, Exchange};

const EXERPT_MAX_LEN: usize = 512;

#[derive(Serialize)]
pub struct ScanResult {
    pub prompt_id: usize,
    pub prompt: String,
    pub response: String,
    pub response_excerpt: String,
    pub safe: bool,
    pub reason: Option<String>,
    pub timestamp: String,
}

impl ScanResult {
    pub fn from_exchange(exchange: &Exchange, evaluation: &Evaluation) -> Self {
        let response_excerpt = if exchange.response.response.len() > EXERPT_MAX_LEN {
            format!("{}...", &exchange.response.response[..EXERPT_MAX_LEN])
        } else {
            exchange.response.response.clone()
        };

        Self {
            prompt_id: exchange.prompt.id,
            prompt: exchange.prompt.prompt.clone(),
            response: exchange.response.response.clone(),
            response_excerpt,
            safe: evaluation.safe,
            reason: evaluation.reason.clone(),
            timestamp: exchange.response.timestamp.clone(),
        }
    }

    pub fn as_jsonl(list: &[Self]) -> anyhow::Result<String> {
        let mut jsonl = String::new();

        for item in list {
            let json = serde_json::to_string(item)?;
            jsonl.push_str(&json);
            jsonl.push('\n');
        }

        Ok(jsonl)
    }

    pub fn as_html(list: &[Self]) -> anyhow::Result<String> {
        let jailbreak_count = list.iter().filter(|r| !r.safe).count();
        let total = list.len();

        let mut html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>LLM Scanner Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        h1 {{ color: #333; border-bottom: 3px solid #007bff; padding-bottom: 10px; }}
        .summary {{ background: #e9ecef; padding: 20px; border-radius: 5px; margin: 20px 0; }}
        .stat {{ display: inline-block; margin-right: 30px; }}
        .stat-label {{ font-weight: bold; color: #666; }}
        .stat-value {{ font-size: 24px; color: #007bff; }}
        .result {{ border: 1px solid #ddd; margin: 20px 0; padding: 20px; border-radius: 5px; }}
        .result.jailbreak {{ border-left: 4px solid #dc3545; background: #fff5f5; }}
        .result.safe {{ border-left: 4px solid #28a745; background: #f5fff5; }}
        .prompt {{ font-weight: bold; color: #333; margin-bottom: 10px; }}
        .response {{ background: #f8f9fa; padding: 15px; border-radius: 4px; margin: 10px 0; font-family: monospace; white-space: pre-wrap; word-wrap: break-word; }}
        .metadata {{ color: #666; font-size: 0.9em; margin-top: 10px; }}
        .reason {{ color: #dc3545; font-weight: bold; margin-top: 10px; }}
        .badge {{ display: inline-block; padding: 4px 12px; border-radius: 12px; font-size: 0.85em; font-weight: bold; }}
        .badge.jailbreak {{ background: #dc3545; color: white; }}
        .badge.safe {{ background: #28a745; color: white; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>LLM Scanner Report</h1>
        <div class="summary">
            <div class="stat">
                <div class="stat-label">Total Prompts</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat">
                <div class="stat-label">Jailbreaks Found</div>
                <div class="stat-value" style="color: #dc3545;">{}</div>
            </div>
            <div class="stat">
                <div class="stat-label">Jailbreak Success Rate</div>
                <div class="stat-value">{:.1}%</div>
            </div>
        </div>
"#,
            total,
            jailbreak_count,
            (jailbreak_count as f64 / total as f64) * 100.0
        );

        for result in list {
            let class = if result.safe { "safe" } else { "jailbreak" };
            let badge_text = if result.safe { "SAFE" } else { "JAILBREAK" };
            let badge_class = if result.safe { "safe" } else { "jailbreak" };

            html.push_str(&format!(
                r#"
        <div class="result {}">
            <span class="badge {}">{}</span>
            <div class="prompt">Prompt #{}: {}</div>
            <div class="response">{}</div>
"#,
                class,
                badge_class,
                badge_text,
                result.prompt_id,
                html_escape(&result.prompt),
                html_escape(&result.response)
            ));

            if let Some(reason) = &result.reason {
                html.push_str(&format!(
                    r#"            <div class="reason">Reason: {}</div>"#,
                    html_escape(reason)
                ));
            }

            html.push_str(&format!(
                r#"            <div class="metadata">Timestamp: {}</div>
        </div>
"#,
                result.timestamp
            ));
        }

        html.push_str(
            r#"    </div>
</body>
</html>"#,
        );

        Ok(html)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
