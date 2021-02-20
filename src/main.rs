mod github;

use github::handle_webhook;
use lambda_runtime::{error::HandlerError, lambda, Context};
use serde::{Deserialize, Serialize};
use simple_error::bail;
use simple_logger::SimpleLogger;
use std::error::Error;

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

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();
    lambda!(handle_proxy_request);

    Ok(())
}

fn handle_proxy_request(r: ProxyRequest, c: Context) -> Result<ProxyResponse, HandlerError> {
    match handle_webhook(serde_json::from_str(&r.body).unwrap()) {
        Ok(_) => Ok(ProxyResponse {
            status_code: 200,
            body: format!(
                "Webhook event has been successfully processed: req_id={} req={:?}",
                c.aws_request_id, r
            ),
        }),
        Err(s) => bail!(format!(
            "Failed to process webhook event: reason={} req_id={} req={:?}",
            s, c.aws_request_id, r
        )
        .as_str()),
    }
}
