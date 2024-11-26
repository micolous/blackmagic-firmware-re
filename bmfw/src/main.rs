#[macro_use]
extern crate tracing;

use crate::error::Error;
use binrw::BinRead;
use clap::{ArgAction, Parser, Subcommand, ValueHint};
use libbmfw::FirmwareFile;
use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{BufReader, Write},
    path::PathBuf,
};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

mod error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Parser)]
struct ExtractOpts {
    /// Firmware file to extract (eg: `data-be74.bin`)
    #[clap(value_hint = ValueHint::FilePath)]
    input: PathBuf,

    /// Directory to extract firmware resources to as `fw-{index}.bin`.
    ///
    /// If the directory or its parents do not exist, the tool will create them.
    ///
    /// If this option is not provided, this tool will just display firmware
    /// headers without extracting them.
    #[clap(short, long, value_hint = ValueHint::DirPath)]
    output_dir: Option<PathBuf>,

    /// Don't automatically decompress supported compressed payloads
    #[clap(long = "no-decompress", action = ArgAction::SetFalse)]
    decompress: bool,

    /// Automatically decompress supported compressed payloads (default)
    #[clap(long = "decompress", overrides_with = "decompress")]
    _no_decompress: bool,
}

#[derive(Debug, Subcommand)]
enum Opt {
    /// Extract a firmware file
    Extract(ExtractOpts),
}

#[derive(Debug, Parser)]
#[clap(verbatim_doc_comment)]
struct CliParser {
    #[clap(subcommand)]
    opt: Opt,
}

fn extract_firmware(o: ExtractOpts) -> Result<()> {
    info!("Extracting firmware payloads from {:?}...", o.input);
    if let Some(od) = &o.output_dir {
        create_dir_all(od)?;
    }

    let mut i = BufReader::new(File::open(o.input)?);
    let firmware = FirmwareFile::read(&mut i)?;

    for (i, rsc) in firmware.resources.iter().enumerate() {
        info!(?rsc);

        if let Some(mut op) = o.output_dir.clone() {
            op.push(format!("fw-{i}.bin"));
            info!("Writing payload to {op:?}...");
            let mut f = OpenOptions::new().write(true).create_new(true).open(op)?;

            if o.decompress && rsc.compression == 1 {
                let mut reader = rsc.decompress_payload();
                std::io::copy(&mut reader, &mut f)?;
            } else {
                if o.decompress && rsc.compression != 0 {
                    warn!("unknown compression type: {}", rsc.compression);
                }
                f.write_all(&rsc.payload)?;
            }

            f.flush()?;
            info!("OK!");
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .compact()
        .init();
    let opts = CliParser::parse();

    match opts.opt {
        Opt::Extract(o) => {
            extract_firmware(o)?;
        }
    }

    Ok(())
}
