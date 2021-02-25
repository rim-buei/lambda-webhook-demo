use crate::github::Client;
use anyhow::{bail, Result};
use ring::hmac;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
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

pub fn handle_request(e: WebhookEvent) -> Result<()> {
    if e.action != "closed" || e.pull_request.is_none() {
        log::info!("ignoring webhook event: event={:?}", e);
        return Ok(());
    }

    handle_pull_request(e.repository, e.pull_request.unwrap())
}

#[derive(Debug, Deserialize)]
struct PullRequest {
    number: u64,
    merged: bool,
    milestone: Option<Milestone>,
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
 * GITHUB_API_ENDPOINT and GITHUB_ACCESS_TOKEN must be set in Lambda.
 */
fn handle_pull_request(repo: Repository, pr: PullRequest) -> Result<()> {
    if !pr.merged || pr.milestone.is_some() || pr.base.reference != repo.default_branch {
        log::info!("ignoring webhook event: type=pull_request pr={:?}", pr);
        return Ok(());
    }
    log::info!("handling webhook event for a pull request: pr={:?}", pr);

    let endpoint = env::var("GITHUB_API_ENDPOINT")?;
    let token = env::var("GITHUB_ACCESS_TOKEN")?;
    let client = Client::new(endpoint, token);

    let milestones: Vec<Milestone> = {
        let path = format!("/repos/{}/milestones?direction=desc", repo.full_name);

        log::info!("sending GET request to GitHub: path={}", path);
        let resp = &client.get(path)?;
        log::info!("retrieved response: {:?}", resp);

        serde_json::from_str(&resp).unwrap()
    };
    if milestones.is_empty() {
        log::info!("milestone is not found: pr={:?}", pr);
        return Ok(());
    }

    let milestone = milestones[0].number;
    {
        let path = format!("/repos/{}/issues/{}", repo.full_name, pr.number);
        let body = json!({ "milestone": milestone });

        log::info!("sending PATCH request to GitHub: path={}", path);
        let resp = &client.patch(path, body)?;
        log::info!("retrieved response: {:?}", resp);
    }

    Ok(())
}

/**
 * This function verifies request body using a secret key string. The secret key
 * string must be configured in both GitHub webhook and Lambda environment variable.
 */
pub fn verify_request(headers: &HashMap<String, String>, body: &str) -> Result<()> {
    if headers.get("X-Hub-Signature-256").is_none() {
        bail!("X-Hub-Signature-256 header not found");
    }
    let gh_sig = headers.get("X-Hub-Signature-256").unwrap();

    log::info!("verifying request body from GitHub: body={}", body);
    let secret = env::var("GITHUB_WEBHOOK_SECRET").unwrap();
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_ref());
    let my_sig = format!(
        "sha256={}",
        hex::encode(hmac::sign(&key, body.as_bytes()).as_ref())
    );

    if *gh_sig != my_sig {
        bail!("invalid request body");
    }

    Ok(())
}
