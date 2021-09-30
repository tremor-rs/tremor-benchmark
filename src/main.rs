use clap::{crate_authors, crate_version, Clap};
use color_eyre::eyre::Result;
use tremor_benchmark::WholeReport;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tremor_benchmark::{convert_into_relevant_data, deserialize, serialize, update_json};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    /// Path to the JSON file where your benchmark data is stored
    data_file: PathBuf,
    /// Path to the JSON file where the last benchmark data is to be stored
    recent_file: PathBuf,
    /// SHA256 of the commit you want to benchmark
    commit_hash: String,
}

fn get_report(commit_hash: &str) -> Result<WholeReport> {
    // calculate short commit hash
    let short_commit_hash = &commit_hash[..6];

    let tag = format!("tremor-benchmark:{}", short_commit_hash);

    // add the commit hash as tag
    Command::new("docker")
        .args(&[
            "build",
            "-t",
            &tag,
            "-f",
            "Dockerfile.bench",
            "--build-arg",
            &format!("commithash={}", commit_hash),
            "docker",
        ])
        .output()?;

    // TODO add some sort of a timeout

    // run benchmarks inside docker image and store it in a report.json
    let r = Command::new("docker").args(["run", &tag]).output()?;
    Command::new("docker")
        .args(&["image", "rm", &tag])
        .output()?;
    deserialize(&r.stdout)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    // TODO check for dependencies like docker, docker-compose and git
    // TODO should be able to use other container runtimes other than docker compose like podman

    let opts: Opts = Opts::parse();
    let report = get_report(&opts.commit_hash)?;

    // parse the report into data.json and recent.json
    fs::write(
        &opts.recent_file,
        serialize(&convert_into_relevant_data(report, &opts.commit_hash)?)?,
    )?;

    fs::write(
        &opts.data_file,
        update_json(
            &fs::read_to_string(&opts.data_file)?,
            &fs::read_to_string(opts.recent_file)?,
        ),
    )?;

    Ok(())
}
