#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![deny(clippy::perf)]
#![deny(clippy::complexity)]
#![deny(clippy::nursery)]
#![deny(clippy::style)]

use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Hello, world!");

    Ok(())
}
