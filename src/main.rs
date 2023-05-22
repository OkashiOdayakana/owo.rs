use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use owo::{shorten, upload};
use std::fs;
use std::io;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// A simple uploader to owo.whats-th.is.
struct Cli {
    /// API Key for owo.
    #[clap(short, long, env = "OWO_KEY")]
    key: String,

    /// File to upload.
    file: Option<PathBuf>,

    /// Domain to display for the end-result file.
    #[clap(short, long, env = "OWO_UPLOAD_URL", default_value = "owo.whats-th.is")]
    result_domain: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Shortens URL
    Shorten { url: String },
}

fn read_up_to(file: &mut impl std::io::Read, mut buf: &mut [u8]) -> Result<usize> {
    let buf_len = buf.len();

    while !buf.is_empty() {
        match file.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(anyhow!(e)),
        }
    }
    Ok(buf_len - buf.len())
}

fn get_reader(file: &Option<PathBuf>) -> Result<Box<dyn Read + Send>> {
    let reader: Box<dyn Read + Send> = match file {
        None => Box::new(io::stdin()),
        Some(filename) => Box::new(fs::File::open(filename)?),
    };

    Ok(reader)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Shorten { url }) => {
            let shorten_result = shorten(&cli.key, &url).context("Failed to shorten!")?;
            println!("{}", shorten_result);
            Ok(())
        }
        None => {
            let mut reader = get_reader(&cli.file).context("Failed to open file")?;
            let mut buffer = [0; 1024];
            read_up_to(&mut reader, &mut buffer)?;

            // Get MIME type and file extension of files.
            let kind = infer::get(&buffer);
            let (mime, filename) = match kind {
                Some(i) => (i.mime_type(), format!("owo.{}", i.extension())),
                None => ("application/octet-stream", String::from("owo")),
            };

            // Chain together the readers, side-stepping the un-seekability of stdin
            let cursor = Cursor::new(buffer);
            let handle = cursor.chain(reader);

            let upload_result = upload(&cli.key, handle, mime, &filename, &cli.result_domain)
                .context("Failed to upload!")?;
            println!("{}", upload_result);
            Ok(())
        }
    }
}
