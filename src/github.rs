use reqwest::header;
use serde::Serialize;

pub struct Client {
    endpoint: String,
    token: String,
}

impl Client {
    pub fn new(endpoint: String, token: String) -> Self {
        Client { endpoint, token }
    }

    pub fn get(&self, path: String) -> Result<String, reqwest::Error> {
        let url = format!("{}{}", self.endpoint, path);
        let client = reqwest::blocking::Client::new();
        client
            .get(&url)
            .header(header::ACCEPT, "application/vnd.github.v3+json")
            .header(header::AUTHORIZATION, self.get_token_header())
            .header(header::USER_AGENT, "rust")
            .send()
            .unwrap()
            .text()
    }

    pub fn patch<T>(&self, path: String, body: T) -> Result<String, reqwest::Error>
    where
        T: Serialize,
    {
        let url = format!("{}{}", self.endpoint, path);
        let client = reqwest::blocking::Client::new();
        client
            .patch(&url)
            .header(header::ACCEPT, "application/vnd.github.v3+json")
            .header(header::AUTHORIZATION, self.get_token_header())
            .header(header::USER_AGENT, "rust")
            .json(&body)
            .send()
            .unwrap()
            .text()
    }

    fn get_token_header(&self) -> String {
        format!("token {}", self.token)
    }
}
