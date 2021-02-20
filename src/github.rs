use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WebhookEvent {
    action: String,
    pull_request: PullRequestEvent,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
    merged: bool,
    milestone: Option<u64>,
}

pub fn handle_webhook(e: WebhookEvent) -> Result<(), String> {
    if e.action != "closed" && !e.pull_request.merged {
        log::info!("Ignorering webhook event: event={:?}", e);
        return Ok(());
    }

    Ok(())
}
