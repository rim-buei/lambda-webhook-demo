use crate::github::Client;
use anyhow::Result;
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Debug, Deserialize)]
pub struct WebhookEvent {
    action: String,
    repository: Repository,

    pull_request: Option<PullRequest>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    full_name: String,
    default_branch: String,
}

pub fn handle(e: WebhookEvent) -> Result<()> {
    if e.action != "closed" || e.pull_request.is_none() {
        log::info!("Ignoring webhook event: event='{:?}'", e);
        return Ok(());
    }

    handle_pull_request(e.repository, e.pull_request.unwrap())
}

#[derive(Debug, Deserialize)]
struct PullRequest {
    number: u64,
    merged: bool,
    milestone: Option<u64>,
    base: Branch,
}

#[derive(Debug, Deserialize)]
struct Branch {
    #[serde(rename = "ref")]
    reference: String,
}

#[derive(Debug, Deserialize)]
struct Milestone {
    number: u64,
}

/**
 * This handler function automatically assigns the latest milestone to a pull
 * request when it is merged into the default branch. Environment variable
 * GITHUB_API_ENDPOINT and GITHUB_ACCESS_TOKEN must be set.
 */
fn handle_pull_request(repo: Repository, pr: PullRequest) -> Result<()> {
    if !pr.merged || pr.milestone.is_some() || pr.base.reference != repo.default_branch {
        log::info!("Ignoring webhook event: type=pull_request, pr={:?}", pr);
        return Ok(());
    }
    log::info!("Handling webhook event for a pull request: pr={:?}", pr);

    let endpoint = env::var("GITHUB_API_ENDPOINT")?;
    let token = env::var("GITHUB_ACCESS_TOKEN")?;
    let client = Client::new(endpoint, token);

    let milestones: Vec<Milestone> = {
        let url = format!("/repos/{}/milestones?direction=desc", repo.full_name);

        log::info!("Sending GET request to GitHub: url={}", url);
        let resp = &client.get(url)?;
        log::info!("Retrieved response: {:?}", resp);

        serde_json::from_str(&resp).unwrap()
    };
    if milestones.len() == 0 {
        log::info!("Milestone is not found: pr={:?}", pr);
        return Ok(());
    }

    let milestone = milestones[0].number;
    {
        let url = format!("/repos/{}/issues/{}", repo.full_name, pr.number);
        let body = json!({ "milestone": milestone });

        log::info!("Sending PATCH request to GitHub: url={}", url);
        let resp = &client.patch(url, body)?;
        log::info!("Retrieved response: {:?}", resp);
    }

    Ok(())
}
