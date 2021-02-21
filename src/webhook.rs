use crate::github::Client;
use anyhow::Result;
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Debug, Deserialize)]
pub struct WebhookEvent {
    action: String,
    repository: Repository,

    pull_request: Option<PullRequestEvent>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    full_name: String,
}

pub fn handle(e: WebhookEvent) -> Result<()> {
    if e.action != "closed" || e.pull_request.is_none() {
        log::info!("Ignoring webhook event: event='{:?}'", e);
        return Ok(());
    }

    handle_pull_request(e.repository, e.pull_request.unwrap())
}

#[derive(Debug, Deserialize)]
struct PullRequestEvent {
    number: u64,
    merged: bool,
    milestone: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct Milestone {
    number: u64,
}

// Automatically assign the latest milestone to a pull request when it is merged
fn handle_pull_request(repo: Repository, e: PullRequestEvent) -> Result<()> {
    if !e.merged || e.milestone.is_some() {
        log::info!("Ignoring webhook event: type=pull_request, event={:?}", e);
        return Ok(());
    }

    let endpoint = env::var("GITHUB_API_ENDPOINT")?;
    let token = env::var("GITHUB_ACCESS_TOKEN")?;
    let client = Client::new(endpoint, token);

    let milestones: Vec<Milestone> = {
        log::debug!("Sending a GET request to GitHub...");
        let url = format!("/repos/{}/milestones?direction=desc", repo.full_name);
        let resp = &client.get(url)?;
        log::debug!("Retrieved response: {:?}", resp);

        serde_json::from_str(&resp).unwrap()
    };
    if milestones.len() == 0 {
        log::info!("Milestone is not found: type=pull_request event={:?}", e);
        return Ok(());
    }

    let milestone = milestones[0].number;
    {
        log::debug!("Sending a PATCH request to GitHub...");
        let url = format!("/repos/{}/issues/{}", repo.full_name, e.number);
        let body = json!({ "milestone": milestone });
        let resp = &client.patch(url, body)?;
        log::debug!("Retrieved response: {:?}", resp);
    }

    Ok(())
}
