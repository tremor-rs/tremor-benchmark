use clap::{crate_authors, crate_version, Clap};
use color_eyre::eyre::Result;

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

fn main() -> Result<()> {
    color_eyre::install()?;
    // TODO check for dependencies like docker, docker-compose

    let opts: Opts = Opts::parse();

    // calculate short commit hash
    let short_commit_hash = &opts.commit_hash[..6];

    let original_working_directory = env::current_dir()?;

    // change the directory to docker/
    // How do I do this? Since I have no idea where my binary would run from. I can assume it runs
    // from the root of the project since that is the most likely scenario but that's not ideal
    env::set_current_dir(original_working_directory.join("docker")).expect("failed to change the current directory to docker/. Are you running this program from the root directory");

    // build Docker Image
    // FIXME this assumes that this process is run from the root directory
    // TODO actually run this
    // add the commit hash as tag
    Command::new("docker")
        .args(&[
            "build",
            ".",
            "-t",
            &format!("tremor-benchmark:{}", short_commit_hash),
        ])
        .output()?;

    // create a report.json file
    // CAVEAT this file can only be created inside the docker/ directory because of our
    // docker-compose.yml
    fs::File::create("report.json")?;
    let report_path = Path::new("report.json").canonicalize()?;

    // write a docker-compose.yml file
    fs::write(
        "docker-compose.yml",
        format!(
            r#"version: '3.8'

services:
  benchmark:
    image: tremor-benchmark:{}
    build: .
    volumes:
      - ./report.json:/report.json
    container_name: tremor-benchmark-compose
"#,
            short_commit_hash
        ),
    )?;

    // run benchmarks inside docker image and store it in a report.json
    Command::new("docker-compose").arg("up").output()?;

    // change the directory back
    env::set_current_dir(original_working_directory)?;

    // parse the report into data.json and recent.json
    fs::write(
        &opts.recent_file,
        serialize(&convert_into_relevant_data(
            deserialize(&fs::read_to_string(report_path)?)?,
            &opts.commit_hash,
        )?)?,
    )?;

    fs::write(
        &opts.data_file,
        update_json(
            &fs::read_to_string(&opts.data_file)?,
            &fs::read_to_string(opts.recent_file)?,
        ),
    )?;

    // TODO remove report.json file

    // TODO remove docker-compose.yml

    // TODO remove docker image

    // TODO remove docker container

    Ok(())
}
