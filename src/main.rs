use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use log;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;

#[derive(Debug, Deserialize)]
struct ProxyRequest {
    body: String,
}

#[derive(Serialize)]
struct ProxyResponse {
    #[serde(rename = "statusCode")]
    status_code: u16,

    body: String,
}

#[derive(Debug, Deserialize)]
struct WebhookEvent {
    action: String,
    pull_request: PullRequestEvent,
}

#[derive(Debug, Deserialize)]
struct PullRequestEvent {
    merged: bool,
    milestone: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();
    lambda!(handle_proxy_event);

    Ok(())
}

fn handle_proxy_event(e: ProxyRequest, c: Context) -> Result<ProxyResponse, HandlerError> {
    handle_event(serde_json::from_str(&e.body).unwrap(), c)
}

fn handle_event(e: WebhookEvent, c: Context) -> Result<ProxyResponse, HandlerError> {
    if e.action != "closed" && !e.pull_request.merged {
        log::info!(
            "Ignorering webhook event: event={:?} request_id={}",
            e,
            c.aws_request_id
        );
        return Ok(ProxyResponse {
            status_code: 200,
            body: format!("Webhook event has been ignored: event={:?}", e),
        });
    }

    Ok(ProxyResponse {
        status_code: 200,
        body: format!("PR has been closed: event={:?}", e),
    })
}
