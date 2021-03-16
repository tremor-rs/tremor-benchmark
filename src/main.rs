#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![deny(clippy::perf)]
#![deny(clippy::complexity)]
#![deny(clippy::nursery)]
#![deny(clippy::style)]

use clap::{crate_authors, crate_version, Clap};
use color_eyre::eyre::Result;

use std::path::PathBuf;

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    root: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opts: Opts = Opts::parse();

    println!("The root directory of tremor is {:?}", opts.root);

    Ok(())
}
