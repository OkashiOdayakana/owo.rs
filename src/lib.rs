use anyhow::{anyhow, Context, Result};
use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};
const API_BASE: &str = "https://api.awau.moe";
const USER_AGENT: &str = "WhatsThisClient (https://owo.codes/okashi/owo-rs, 0.3.1)";

#[derive(Debug, Serialize, Deserialize)]
pub struct APIResponse {
    pub success: bool,
    pub files: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub success: bool,
    pub hash: String,
    pub name: String,
    pub url: String,
    pub size: Option<i64>,
}

/// Use the whats-th.is API to shorten a link.
///
/// # Arguments
///
/// * `key` - A valid whats-th.is API token.
///
/// * `s_url` - The URL to be shortened.
pub fn shorten(key: &str, s_url: &str) -> Result<String> {
    let url = format!("{API_BASE}/shorten/polr");

    let client = reqwest::blocking::Client::new();
    let params = [("action", "shorten"), ("url", s_url)];
    let url = reqwest::Url::parse_with_params(&url, params)?;

    let resp = client
        .get(url)
        .header(reqwest::header::AUTHORIZATION, key)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?;

    match resp.status() {
        reqwest::StatusCode::OK => Ok(resp.text()?),
        i => Err(anyhow!(i)),
    }
}

/// Uploads a file with the whats-th.is API.
///
/// TODO: Enable concurrent uploads with a single upload
///
/// # Arguments
/// * `key` - A valid whats-th.is API token.
///
/// * `in_file` - The file to be uploaded. Anything that implements std::io::Read should do.
///
/// * `mime_type` - A valid mime type, e.g "image/png".
///
/// * `file_name` - The desired upload file name.
///
/// * `result_url` - The vanity domain you wish to use.
pub fn upload<
    R: std::io::Read + std::marker::Send + 'static,
    S: Into<String> + std::fmt::Display,
>(
    key: &str,
    in_file: R,
    mime_type: S,
    file_name: S,
    result_url: S,
) -> Result<String> {
    let mime_type = mime_type.into();
    let file_name = file_name.into();

    let url = format!("{API_BASE}/upload/pomf");

    let file_part = multipart::Part::reader(in_file)
        .file_name(file_name)
        .mime_str(&mime_type)
        .expect("Error creating Multiform Part!");

    let multi_form = multipart::Form::new()
        .text("type", mime_type)
        .part("files[]", file_part);

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(url)
        .header(reqwest::header::AUTHORIZATION, key)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .multipart(multi_form)
        .send()
        .context("Failed to get request")?;

    match resp.status() {
        reqwest::StatusCode::OK => match resp.json::<APIResponse>() {
            Ok(i) => Ok(format!("https://{}/{}", result_url, i.files[0].url)),
            Err(e) => Err(anyhow!(e)),
        },
        i => Err(anyhow!(i)),
    }
}
