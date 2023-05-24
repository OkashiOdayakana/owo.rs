use anyhow::{anyhow, Context, Result};
use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};
const API_BASE: &str = "https://api.awau.moe";
const USER_AGENT: &str = "WhatsThisClient (https://owo.codes/okashi/owo-rs, 0.4.0)";

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub data: FileListData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListResponse {
    pub success: bool,
    pub total_objects: i64,
    pub data: Vec<FileListData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListData {
    pub bucket: String,
    pub key: String,
    pub dir: String,
    pub r#type: i64,
    pub dest_url: Option<String>,
    pub content_type: Option<String>,
    pub content_length: Option<i64>,
    pub created_at: String,
    pub deleted_at: Option<String>,
    pub delete_reason: Option<String>,
    pub md5_hash: Option<String>,
    pub sha256_hash: Option<String>,
    pub associated_with_current_user: bool,
}

/// View files associated with your whats-th.is account.
///
/// Paginated, with `entries` amount of listings from `offset`.
///
/// # Arguments
///
/// * `key` - A valid whats-th.is API token.
///
/// * `entries` - The amount of entries to display at once.
///
/// * `offset` - The offset from which to display, with `0` being the most recently uploaded file.
///
pub fn list_files(key: &str, entries: &i64, offset: &i64) -> Result<FileListResponse> {
    let url = format!("{API_BASE}/objects?limit={entries}&offset={offset}");

    let client = reqwest::blocking::Client::new();

    let resp = client
        .get(url)
        .header(reqwest::header::AUTHORIZATION, key)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?;

    match resp.status() {
        reqwest::StatusCode::OK => Ok(resp.json::<FileListResponse>()?),
        i => Err(anyhow!(i)),
    }
}

/// Delete a file associated with your whats-th.is account.
///
/// Must have been associated (i.e uploaded via /upload/pomf/associated) to be deletable
///
/// Works on both files and redirects (shortened URLs.)
///
/// # Arguments
///
/// * `key` - A valid whats-th.is API token.
///
/// * `object` - The object you are trying to delete, without the domain. (i.e  "/7IqAPwr")
pub fn delete_file(key: &str, object: &str) -> Result<DeleteResponse> {
    let url = format!("{API_BASE}/objects/{object}");

    let client = reqwest::blocking::Client::new();

    let resp = client
        .delete(url)
        .header(reqwest::header::AUTHORIZATION, key)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?;

    match resp.status() {
        reqwest::StatusCode::OK => Ok(resp.json::<DeleteResponse>()?),
        i => Err(anyhow!(i)),
    }
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
    associated: &bool,
) -> Result<UploadResponse> {
    let mime_type = mime_type.into();
    let file_name = file_name.into();

    let url = match associated {
        true => format!("{}/upload/pomf/associated", API_BASE),
        false => format!("{}/upload/pomf", API_BASE),
    };

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
        reqwest::StatusCode::OK => match resp.json::<UploadResponse>() {
            Ok(i) => Ok(i),
            Err(e) => Err(anyhow!(e)),
        },
        i => Err(anyhow!(i)),
    }
}
