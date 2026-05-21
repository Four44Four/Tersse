//! HTTPS streaming client for Google Gemini `streamGenerateContent` (SSE).

use crate::constants::{gemini_model, GEMINI_API_BASE};
use reqwest::blocking::{Client, Response};
use reqwest::StatusCode;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

const MAX_RETRIES: u32 = 4;
const INITIAL_BACKOFF: Duration = Duration::from_secs(2);

#[derive(Debug, Clone)]
pub enum AiStreamEvent {
    Token(String),
    Done,
    Error(String),
}

/// Spawn a background thread that streams model tokens over `Receiver`.
pub fn start_stream(api_key: &str, prompt: &str) -> Receiver<AiStreamEvent> {
    let (tx, rx) = mpsc::channel();
    let api_key = api_key.to_string();
    let prompt = prompt.to_string();
    thread::spawn(move || {
        if let Err(e) = stream_generate_content(&api_key, &prompt, &tx) {
            let _ = tx.send(AiStreamEvent::Error(e));
        } else {
            let _ = tx.send(AiStreamEvent::Done);
        }
    });
    rx
}

fn stream_url(model: &str) -> String {
    format!(
        "{}/models/{}:streamGenerateContent?alt=sse",
        GEMINI_API_BASE, model
    )
}

fn request_body(prompt: &str) -> Value {
    json!({
        "contents": [{
            "parts": [{ "text": prompt }]
        }]
    })
}

fn post_stream(
    client: &Client,
    url: &str,
    api_key: &str,
    body: &Value,
) -> Result<Response, String> {
    client
        .post(url)
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(body)
        .send()
        .map_err(|e| e.to_string())
}

fn retry_delay(response: &Response, attempt: u32) -> Duration {
    if let Some(value) = response.headers().get("retry-after") {
        if let Ok(s) = value.to_str() {
            if let Ok(secs) = s.parse::<u64>() {
                return Duration::from_secs(secs.max(1));
            }
        }
    }
    INITIAL_BACKOFF * 2u32.saturating_pow(attempt)
}

fn format_api_error(status: StatusCode, body: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        if let Some(msg) = value
            .pointer("/error/message")
            .and_then(|m| m.as_str())
        {
            return format!("API error {status}: {msg}");
        }
    }
    format!("API error {status}: {body}")
}

fn post_with_retry(
    client: &Client,
    url: &str,
    api_key: &str,
    body: &Value,
) -> Result<Response, String> {
    let mut attempt = 0;
    loop {
        let response = post_stream(client, url, api_key, body)?;

        if response.status() == StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
            thread::sleep(retry_delay(&response, attempt));
            attempt += 1;
            continue;
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().unwrap_or_default();
            return Err(format_api_error(status, &text));
        }

        return Ok(response);
    }
}

fn stream_generate_content(
    api_key: &str,
    prompt: &str,
    tx: &Sender<AiStreamEvent>,
) -> Result<(), String> {
    let model = gemini_model();
    let url = stream_url(&model);
    let body = request_body(prompt);

    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let response = post_with_retry(&client, &url, api_key, &body)?;

    let reader = BufReader::new(response);
    let mut sent_any = false;

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let payload = line.strip_prefix("data: ").unwrap_or(&line).trim();
        if payload.is_empty() || payload == "[DONE]" {
            continue;
        }

        if let Some(text) = extract_text(payload) {
            if !text.is_empty() {
                sent_any = true;
                tx.send(AiStreamEvent::Token(text))
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    if !sent_any {
        return Err("stream ended with no tokens".to_string());
    }

    Ok(())
}

fn extract_text(json_str: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let parts = value
        .pointer("/candidates/0/content/parts")
        .and_then(|p| p.as_array())?;
    let mut out = String::new();
    for part in parts {
        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
            out.push_str(text);
        }
    }
    if out.is_empty() { None } else { Some(out) }
}
