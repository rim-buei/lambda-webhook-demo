mod github;
mod webhook;

use lambda_runtime::{error::HandlerError, lambda, Context};
use serde::{Deserialize, Serialize};
use simple_error::bail;
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct ProxyRequest {
    headers: HashMap<String, String>,
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
    if let Err(e) = webhook::verify_request(&r.headers, &r.body) {
        return Ok(ProxyResponse {
            status_code: 403,
            body: format!(
                "your request could not be processed: reason={} req_id={} req={:?}",
                e, c.aws_request_id, r
            ),
        });
    }

    match webhook::handle_request(serde_json::from_str(&r.body).unwrap()) {
        Ok(_) => Ok(ProxyResponse {
            status_code: 200,
            body: format!(
                "webhook event has been successfully processed: req_id={} req={:?}",
                c.aws_request_id, r
            ),
        }),
        Err(e) => bail!(format!(
            "failed to process webhook event: reason={} req_id={} req={:?}",
            e, c.aws_request_id, r
        )
        .as_str()),
    }
}
