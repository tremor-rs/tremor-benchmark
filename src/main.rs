use clap::{crate_authors, crate_version, Clap};
use color_eyre::eyre::Result;
use dotenv::dotenv;

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use tremor_benchmark::{convert_into_relevant_data, deserialize, serialize, update_json};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    /// SHA256 of the commit you want to benchmark
    commit_hash: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    // TODO check for dependencies like docker, docker-compose and git
    // TODO should be able to use other container runtimes other than docker like podman

    dotenv().ok();

    let opts: Opts = Opts::parse();

    // calculate short commit hash
    let short_commit_hash = &opts.commit_hash[..6];

    let original_working_directory = env::current_dir()?;

    // change the directory to docker/
    // How do I do this? Since I have no idea where my binary would run from. I can assume it runs
    // from the root of the project since that is the most likely scenario but that's not ideal
    env::set_current_dir(original_working_directory.join("docker")).expect("failed to change the current directory to docker/. Are you running this program from the root directory");

    // TODO disable networking in the docker container

    // build Docker Image
    // FIXME this assumes that this process is run from the docker/ directory
    fs::write(
        "Dockerfile",
        include_str!("../docker/Dockerfile.in").replace("COMMITHASH", &opts.commit_hash),
    )?;

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
    // TODO read the docker-compose.yml file to string
    // TODO use the docker copy command to copy the report.json file instead of doing this
    fs::write(
        "docker-compose.yml",
        format!(
            r#"version: '3.3'

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

    // TODO add some sort of a timeout

    // run benchmarks inside docker image and store it in a report.json
    Command::new("docker-compose").arg("up").output()?;

    // change the directory back
    env::set_current_dir(&original_working_directory)?;

    // push the data to the github repo
    Command::new("git")
        .args(&[
            "clone",
            "--depth=1",
            &format!(
                "https://{}@github.com/tremor-rs/tremor-benchmark.git",
                env::var("TREMORBOT_PAT")?
            ),
            "tremor-benchmark-cloned",
        ])
        .output()?;

    env::set_current_dir("tremor-benchmark-cloned")
        .expect("failed to change the current directory to tremor-benchmark-cloned");

    // parse the report into data.json and recent.json
    fs::write(
        Path::new("data").join("recent.json"),
        serialize(&convert_into_relevant_data(
            deserialize(&fs::read_to_string(report_path)?)?,
            &opts.commit_hash,
        )?)?,
    )?;

    fs::write(
        Path::new("data").join("data.json"),
        update_json(
            &fs::read_to_string(Path::new("data").join("data.json"))?,
            &fs::read_to_string(Path::new("data").join("recent.json"))?,
        ),
    )?;

    // git config user.email 81628356+tremorbot@users.noreply.github.com
    Command::new("git")
        .args(&[
            "config",
            "user.email",
            "81628356+tremorbot@users.noreply.github.com",
        ])
        .output()?;
    // git config user.name "tremorbot"
    Command::new("git")
        .args(&["config", "user.name", "tremorbot"])
        .output()?;
    // read TREMORBOT_PAT from environment
    // git add data/data.json data/recent.json
    Command::new("git")
        .args(&["add", "data/data.json", "data/recent.json"])
        .output()?;
    // git commit --message "chore(data): update benchmarks for SHORTCOMMITHASH"
    Command::new("git")
        .args(&[
            "commit",
            "-m",
            &format!("chore(data): update benchmarks for {}", opts.commit_hash),
        ])
        .output()?;
    // git push
    Command::new("git").arg("push").output()?;

    // change the directory back
    env::set_current_dir(&original_working_directory)?;

    // remove report.json file
    fs::remove_file(Path::new("docker").join("Dockerfile"))?;

    // remove report.json file
    fs::remove_file(Path::new("docker").join("report.json"))?;

    // remove docker-compose.yml
    fs::remove_file(Path::new("docker").join("docker-compose.yml"))?;

    fs::remove_dir_all("tremor-benchmark-cloned")?;

    // this is probably very bad
    // FIXME this doesn't really work because docker system prune --all prompts
    // for a confirmation
    Command::new("docker")
        .args(&["system", "prune", "--all"])
        .output()?;

    Ok(())
}
