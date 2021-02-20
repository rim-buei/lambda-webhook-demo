use serde::Deserialize;

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

pub fn handle_webhook(e: WebhookEvent) -> Result<(), String> {
    if e.action != "closed" || e.pull_request.is_none() {
        log::info!("Ignoring webhook event: event={:?}", e);
        return Ok(());
    }

    Ok(())
}
