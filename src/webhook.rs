use crate::github::Client;
use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct WebhookEvent {
    action: String,
    pull_request: Option<PullRequestEvent>,
}

#[derive(Debug, Deserialize)]
struct PullRequestEvent {
    merged: bool,
    milestone: Option<u64>,
}

pub fn handle(e: WebhookEvent) -> Result<()> {
    if e.action != "closed" || e.pull_request.is_none() {
        log::info!("Ignoring webhook event: event={:?}", e);
        return Ok(());
    }

    handle_pull_request(e.pull_request.unwrap())
}

#[derive(Debug, Deserialize)]
struct Milestone {
    number: u64,
}

fn handle_pull_request(e: PullRequestEvent) -> Result<()> {
    if !e.merged || e.milestone.is_some() {
        log::info!("Ignoring webhook event: type=pull_request event={:?}", e);
        return Ok(());
    }

    let endpoint = env::var("GITHUB_API_ENDPOINT")?;
    let token = env::var("GITHUB_ACCESS_TOKEN")?;

    let client = Client::new(endpoint, token);
    let milestones =
        client.get("/repos/rim-buei/lambda-webhook-demo/milestones?direction=asc".to_string())?;
    log::info!("Milestones: {}", milestones);

    Ok(())
}
