mod github;
mod webhook;

use lambda_runtime::{handler_fn, Context, Error};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use simple_error::bail;
use simple_logger::SimpleLogger;
use std::collections::HashMap;

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let func = handler_fn(handle_proxy_request);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn handle_proxy_request(
    r: ProxyRequest,
    c: Context,
) -> Result<ProxyResponse, Error> {
    if let Err(e) = webhook::verify_request(&r.headers, &r.body) {
        return Ok(ProxyResponse {
            status_code: 403,
            body: format!(
                "your request could not be processed: reason={} req_id={} req={:?}",
                e, c.request_id, r
            ),
        });
    }

    match webhook::handle_request(serde_json::from_str(&r.body).unwrap()) {
        Ok(_) => Ok(ProxyResponse {
            status_code: 200,
            body: format!(
                "webhook event has been successfully processed: req_id={} req={:?}",
                c.request_id, r
            ),
        }),
        Err(e) => bail!(format!(
            "failed to process webhook event: reason={} req_id={} req={:?}",
            e, c.request_id, r
        )
        .as_str()),
    }
}
