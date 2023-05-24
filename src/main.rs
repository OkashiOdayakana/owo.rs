use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand};
use owo::{delete_file, list_files, shorten, upload};
use std::fs;
use std::io;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// A simple uploader to owo.whats-th.is.
struct Cli {
    #[command(flatten)]
    globopts: GlobalOpts,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Args, Debug)]
struct GlobalOpts {
    #[clap(short, long, env = "OWO_KEY")]
    key: String,
}

#[derive(Subcommand)]
#[clap(infer_subcommands = true, subcommand_required = true)]
enum Commands {
    /// Upload a file
    Upload {
        /// Set whether to associate the upload with your account.
        #[clap(short, long, action, env = "OWO_ASSOCIATED", default_value_t = false)]
        associated: bool,

        /// File to upload.
        file: Option<PathBuf>,

        /// The domain to use to display the uploaded file.
        ///
        /// Defaults to owo.whats-th.is.
        #[clap(
            short,
            long,
            env = "OWO_RESULT_DOMAIN",
            default_value = "owo.whats-th.is"
        )]
        result_domain: String,
    },
    /// Shortens URL
    Shorten { url: String },
    /// Lists Associated Files
    ListFiles {
        #[clap(short, long, default_value = "8")]
        entries: i64,

        #[clap(short, long, default_value = "0")]
        offset: i64,
    },
    /// Delete an object from OwO.
    Delete { object: String },
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
            let shorten_result = shorten(&cli.globopts.key, &url).context("Failed to shorten!")?;
            println!("{}", shorten_result);
            Ok(())
        }
        Some(Commands::ListFiles { entries, offset }) => {
            let files_result = list_files(&cli.globopts.key, &entries, &offset)?;
            println!("Showing {entries} entries, from offset {offset}\n");

            for i in files_result.data {
                let (key, created_at) = (i.key, i.created_at);
                match i.r#type {
                    0 => {
                        let (content_type, content_length, md5_hash) = (
                            i.content_type.unwrap(),
                            i.content_length.unwrap(),
                            i.md5_hash.unwrap(),
                        );
                        print!("\nType: File\nKey: {}\nCreation Date: {}\nMIME Type: {}\nFile Length: {}\nMD5 Hash: {}\n", 
                            key, created_at, content_type, content_length, md5_hash);
                    }
                    1 => {
                        let dest_url = i.dest_url.unwrap();
                        print!(
                            "\nType: Redirect\nKey: {}\nRedirect URL: {}\nCreation Date: {}\n",
                            key, dest_url, created_at
                        );
                    }
                    2 => {
                        let (deleted_at, delete_reason) =
                            (i.deleted_at.unwrap(), i.delete_reason.unwrap());

                        print!("\nType: Tombstone\nKey: {}\nCreation Date: {}\nDeletion Date: {}\nReason For Deletion: {}\n", 
                            key, created_at, deleted_at, delete_reason);
                    }

                    _ => {}
                }
            }
            Ok(())
        }
        Some(Commands::Upload {
            associated,
            file,
            result_domain,
        }) => {
            let mut reader = get_reader(&file).context("Failed to open file")?;
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

            let upload_result = upload(&cli.globopts.key, handle, mime, &filename, &associated)
                .context("Failed to upload!")?;
            println!("https://{}/{}", result_domain, upload_result.files[0].url);
            Ok(())
        }
        Some(Commands::Delete { object }) => {
            let delete_result = delete_file(&cli.globopts.key, &object)?;
            println!(
                "Success! Object {} deleted at {}",
                delete_result.data.key,
                delete_result.data.deleted_at.unwrap()
            );
            Ok(())
        }
        None => Err(anyhow!("You shouldn't be here!")),
    }
}
