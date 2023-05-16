use clap::{Parser, Subcommand};
use owo_rs::{shorten, upload};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// A simple uploader to owo.whats-th.is.
struct Cli {
    /// API Key for owo.
    #[arg(short, long)]
    key: String,

    /// File to upload.
    file: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Shortens URL
    Shorten { url: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Shorten { url }) => match shorten(&cli.key, &url) {
            Ok(i) => println!("{}", i),
            Err(e) => eprintln!("{}", e),
        },
        None => {
            let mut reader: Box<dyn BufRead> = match cli.file {
                None => Box::new(BufReader::new(io::stdin())),
                Some(filename) => Box::new(BufReader::new(File::open(filename).unwrap())),
            };

            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).expect("Failed to read file!");

            match upload(&cli.key, buf) {
                Ok(i) => println!("{}", i),
                Err(e) => println!("{}", e),
            }
        }
    }
}
