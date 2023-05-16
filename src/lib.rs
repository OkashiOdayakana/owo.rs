use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};
use std::error::Error;
const API_BASE: &str = "https://api.awau.moe";
const USER_AGENT: &str = "WhatsThisClient (https://owo.codes/okashi/owo-rs, 0.0.1)";

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
    pub size: i64,
}

/// Use the whats-th.is API to shorten a link.
///
/// # Arguments
///
/// * `key` - A valid whats-th.is API token.
///
/// * `s_url` - The URL to be shortened.
pub fn shorten(key: &str, s_url: &str) -> Result<String, Box<dyn Error>> {
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
        reqwest::StatusCode::UNAUTHORIZED => Err("Invalid OwO token!".into()),
        i => Err(format!("Received status code: {}", i).into()),
    }
}

// Uploads a file with the whats-th.is API.
//
// TODO: Enable concurrent uploads with a single upload
//
// # Arguments
// * `key` - A valid whats-th.is API token.
//
// * `in_file` - A file, expressed as a `Vec<u8>`
pub fn upload(key: &str, in_file: Vec<u8>) -> Result<String, Box<dyn Error>> {
    let url = format!("{API_BASE}/upload/pomf");

    let kind = infer::get(&in_file);
    let mime = match kind {
        Some(i) => i.mime_type(),
        None => "application/octet-stream",
    };
    let filename = match kind {
        Some(i) => format!("owo.{}", i.extension()),
        None => String::from("owo"),
    };

    let file_part = multipart::Part::bytes(in_file)
        .file_name(filename)
        .mime_str(mime)
        .expect("Error creating Multiform Part!");

    let multi_form = multipart::Form::new()
        .text("type", mime)
        .part("files[]", file_part);

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(url)
        .header(reqwest::header::AUTHORIZATION, key)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .multipart(multi_form)
        .send()?;

    match resp.status() {
        reqwest::StatusCode::OK => match resp.json::<APIResponse>() {
            Ok(i) => Ok(format!("https://owo.whats-th.is/{}", i.files[0].url)),
            Err(_) => Err("Unable to parse JSON!".into()),
        },
        reqwest::StatusCode::UNAUTHORIZED => Err("Invalid OwO token!".into()),
        i => Err(format!("Received status code: {}", i).into()),
    }
}
