use chrono::offset::Utc;
use color_eyre::eyre::Result;
use serde::Deserialize;
use serde::Serialize;

use std::fs;
use std::path::Path;
use std::process::Command;

fn update_json(old: &str, to_be_added: &str) -> String {
    // old is of the format [{},{},{}]
    // new is of the format {}
    // output should be [{},{}.{},{}]
    let last_character_removed = old.trim().strip_suffix(']').unwrap();
    let final_json = format!("{},{}]", last_character_removed, to_be_added);
    final_json
}

// TODO The name is horrible here. pls help
#[derive(Deserialize, Debug)]
pub struct WholeReport {
    metadata: Metadata,
    includes: Vec<String>,
    excludes: Vec<String>,
    reports: Reports,
    stats: Stats,
}

#[derive(Deserialize, Debug)]
struct Reports {
    bench: Vec<SingleReport>,
}

#[derive(Deserialize, Debug)]
struct SingleReport {
    description: String,
    elements: Element,
}

#[derive(Deserialize, Debug)]
struct Element {
    bench: Bench,
}

#[derive(Deserialize, Debug)]
struct Bench {
    name: String,
    description: String,
    elements: Vec<String>,
    evidence: Evidence,
    stats: Stat,
    duration: usize,
}

#[derive(Deserialize, Debug)]
struct Evidence {
    #[serde(rename = "test: stdout")]
    stdout: String,
    #[serde(rename = "test: stderr")]
    stderr: String,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    allocator: String,
    repository: String,
    description: String,
    homepage: String,
    name: String,
    authors: String,
    librdkafka: String,
    version: String,
}

#[derive(Deserialize, Debug)]
struct Stats {
    command: Stat,
    all: Stat,
    integration: Stat,
    unit: Stat,
    bench: Stat,
}

#[derive(Deserialize, Debug)]
struct Stat {
    pass: u16,
    fail: u16,
    skip: u16,
    assert: u16,
}

// TODO Add test
pub fn deserialize(report: &str) -> Result<WholeReport> {
    let report: WholeReport = serde_json::from_str(report)?;

    Ok(report)
}

// Here path is the path of the git directory
fn find_commit_hash<S>(dir: S) -> Result<String>
where
    S: AsRef<Path>,
{
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()?
        .stdout;
    let output = std::str::from_utf8(&output)?.trim();
    Ok(output.into())
}

fn date_and_time() -> String {
    Utc::now().to_string()
}

#[derive(Serialize, Debug)]
pub struct Data {
    created_at: String,
    commit_hash: String,
    benchmarks: Vec<Benchmark>,
}

impl Data {
    fn new() -> Self {
        todo!()
    }
}

#[derive(Serialize, Debug)]
struct Benchmark {
    name: String,
    throughput: f64,
}

pub fn convert_into_relevant_data(whole_report: WholeReport) -> Result<Data> {
    let mut benchmarks: Vec<Benchmark> = vec![];
    for report in whole_report.reports.bench {
        // TODO add a check if benchmark has passed or failed
        let name = report.elements.bench.name;
        let throughput = extract_throughput(&report.elements.bench.evidence.stdout).unwrap();
        benchmarks.push(Benchmark { name, throughput });
    }
    let commit_hash = find_commit_hash("/home/humancalico/tremor-runtime")?;
    let created_at = date_and_time();
    Ok(Data {
        created_at,
        commit_hash,
        benchmarks,
    })
}

pub fn serialize(data: &Data) -> Result<String> {
    Ok(serde_json::to_string(data)?)
}

// Run the benchmarks and store them in a report.json
// TODO Write tests for this
fn run_benchmark<S>(project_root: S) -> Result<String>
where
    S: AsRef<Path>,
{
    run_benchmark_with_tags(project_root, &[])
}

// Currently there is no good way to run benchmarks knowing only their name so tags are used.
// This should be a feature in the upstream CLI
// This function compares the tags and runs the benchmark only if all the necessary tags are
// present
pub fn run_benchmark_with_tags<S>(project_root: S, tags: &[&str]) -> Result<String>
where
    S: AsRef<Path>,
{
    let arguments = [
        &["test", "bench", "tremor-cli/tests/bench", "-o", "-i"],
        tags,
    ]
    .concat();

    // ./target/release/tremor test bench tremor-cli/tests/bench/
    // The -o flag stores the benchmarks in a report.json
    Command::new("./target/release/tremor")
        .args(arguments)
        .current_dir(project_root)
        .output()?;

    Ok(fs::read_to_string("report.json")?)
}

fn extract_raw_histogram(log_string: &str) -> Option<&str> {
    let end = log_string.find("Throughput:").unwrap();

    if let Some(raw_histogram) = log_string.get(0..end) {
        Some(raw_histogram.trim())
    } else {
        None
    }
}

fn extract_throughput(log_string: &str) -> Option<f64> {
    let start = log_string.find("Throughput:").unwrap() + 12;
    let end = log_string.find("MB/s").unwrap();

    if let Some(throughput_string) = log_string.get(start..end) {
        Some(throughput_string.trim().parse().unwrap())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_extract_raw_histogram() {
        let some_log = r#"
     Value Percentile TotalCount 1/(1-Percentile)

        1343 0.00000          1           1.00
        6495 0.25000   21519175           1.33
        8895 0.50000   43774741           2.00
       10111 0.62500   53834372           2.67
       21247 0.75000   64466758           4.00
       31615 0.81250   69841533           5.33
       44031 0.87500   75268579           8.00
       51199 0.90625   77931574          10.67
       60159 0.93750   80644651          16.00
       65279 0.95312   81924551          21.33
       73215 0.96875   83285642          32.00
       79871 0.97656   83938098          42.67
       91135 0.98438   84624143          64.00
       99839 0.98828   84958398          85.33
      116735 0.99219   85280988         128.00
      133119 0.99414   85457548         170.67
      153599 0.99609   85618379         256.00
      163839 0.99707   85709966         341.33
      172031 0.99805   85789891         512.00
      177151 0.99854   85827174         682.67
      189439 0.99902   85869533        1024.00
      203775 0.99927   85889457        1365.33
      228351 0.99951   85910431        2048.00
      246783 0.99963   85921094        2730.67
      270335 0.99976   85931344        4096.00
      288767 0.99982   85936712        5461.33
      319487 0.99988   85941757        8192.00
      342015 0.99991   85944442       10922.67
      376831 0.99994   85947012       16384.00
      397311 0.99995   85948245       21845.33
      448511 0.99997   85949562       32768.00
      473087 0.99998   85950208       43690.67
      540671 0.99998   85950877       65536.00
      561151 0.99999   85951202       87381.33
      638975 0.99999   85951514      131072.00
      679935 0.99999   85951685      174762.67
      724991 1.00000   85951874      262144.00
      749567 1.00000   85951940      349525.33
      806911 1.00000   85951999      524288.00
      815103 1.00000   85952066      699050.67
      819199 1.00000   85952092     1048576.00
      831487 1.00000   85952103     1398101.33
      851967 1.00000   85952128     2097152.00
      856063 1.00000   85952133     2796202.67
      880639 1.00000   85952144     4194304.00
      884735 1.00000   85952151     5592405.33
      884735 1.00000   85952151     8388608.00
      888831 1.00000   85952159    11184810.67
      888831 1.00000   85952159    16777216.00
      888831 1.00000   85952159    22369621.33
      888831 1.00000   85952159    33554432.00
      892927 1.00000   85952161    44739242.67
      892927 1.00000   85952161            inf
#[Mean       =     18564.86, StdDeviation   =     23434.73]
#[Max        =       892927, Total count    =     85952161]
#[Buckets    =           30, SubBuckets     =         3968]


Throughput: 515.7 MB/s
            "#;

        let raw_histogram = Some(
            r#"
     Value Percentile TotalCount 1/(1-Percentile)

        1343 0.00000          1           1.00
        6495 0.25000   21519175           1.33
        8895 0.50000   43774741           2.00
       10111 0.62500   53834372           2.67
       21247 0.75000   64466758           4.00
       31615 0.81250   69841533           5.33
       44031 0.87500   75268579           8.00
       51199 0.90625   77931574          10.67
       60159 0.93750   80644651          16.00
       65279 0.95312   81924551          21.33
       73215 0.96875   83285642          32.00
       79871 0.97656   83938098          42.67
       91135 0.98438   84624143          64.00
       99839 0.98828   84958398          85.33
      116735 0.99219   85280988         128.00
      133119 0.99414   85457548         170.67
      153599 0.99609   85618379         256.00
      163839 0.99707   85709966         341.33
      172031 0.99805   85789891         512.00
      177151 0.99854   85827174         682.67
      189439 0.99902   85869533        1024.00
      203775 0.99927   85889457        1365.33
      228351 0.99951   85910431        2048.00
      246783 0.99963   85921094        2730.67
      270335 0.99976   85931344        4096.00
      288767 0.99982   85936712        5461.33
      319487 0.99988   85941757        8192.00
      342015 0.99991   85944442       10922.67
      376831 0.99994   85947012       16384.00
      397311 0.99995   85948245       21845.33
      448511 0.99997   85949562       32768.00
      473087 0.99998   85950208       43690.67
      540671 0.99998   85950877       65536.00
      561151 0.99999   85951202       87381.33
      638975 0.99999   85951514      131072.00
      679935 0.99999   85951685      174762.67
      724991 1.00000   85951874      262144.00
      749567 1.00000   85951940      349525.33
      806911 1.00000   85951999      524288.00
      815103 1.00000   85952066      699050.67
      819199 1.00000   85952092     1048576.00
      831487 1.00000   85952103     1398101.33
      851967 1.00000   85952128     2097152.00
      856063 1.00000   85952133     2796202.67
      880639 1.00000   85952144     4194304.00
      884735 1.00000   85952151     5592405.33
      884735 1.00000   85952151     8388608.00
      888831 1.00000   85952159    11184810.67
      888831 1.00000   85952159    16777216.00
      888831 1.00000   85952159    22369621.33
      888831 1.00000   85952159    33554432.00
      892927 1.00000   85952161    44739242.67
      892927 1.00000   85952161            inf
#[Mean       =     18564.86, StdDeviation   =     23434.73]
#[Max        =       892927, Total count    =     85952161]
#[Buckets    =           30, SubBuckets     =         3968]
            "#
            .trim(),
        );

        assert_eq!(extract_raw_histogram(some_log), raw_histogram);
    }

    #[test]
    fn test_extract_throughput() {
        let some_log = r#"
     Value Percentile TotalCount 1/(1-Percentile)

        1343 0.00000          1           1.00
        6495 0.25000   21519175           1.33
        8895 0.50000   43774741           2.00
       10111 0.62500   53834372           2.67
       21247 0.75000   64466758           4.00
       31615 0.81250   69841533           5.33
       44031 0.87500   75268579           8.00
       51199 0.90625   77931574          10.67
       60159 0.93750   80644651          16.00
       65279 0.95312   81924551          21.33
       73215 0.96875   83285642          32.00
       79871 0.97656   83938098          42.67
       91135 0.98438   84624143          64.00
       99839 0.98828   84958398          85.33
      116735 0.99219   85280988         128.00
      133119 0.99414   85457548         170.67
      153599 0.99609   85618379         256.00
      163839 0.99707   85709966         341.33
      172031 0.99805   85789891         512.00
      177151 0.99854   85827174         682.67
      189439 0.99902   85869533        1024.00
      203775 0.99927   85889457        1365.33
      228351 0.99951   85910431        2048.00
      246783 0.99963   85921094        2730.67
      270335 0.99976   85931344        4096.00
      288767 0.99982   85936712        5461.33
      319487 0.99988   85941757        8192.00
      342015 0.99991   85944442       10922.67
      376831 0.99994   85947012       16384.00
      397311 0.99995   85948245       21845.33
      448511 0.99997   85949562       32768.00
      473087 0.99998   85950208       43690.67
      540671 0.99998   85950877       65536.00
      561151 0.99999   85951202       87381.33
      638975 0.99999   85951514      131072.00
      679935 0.99999   85951685      174762.67
      724991 1.00000   85951874      262144.00
      749567 1.00000   85951940      349525.33
      806911 1.00000   85951999      524288.00
      815103 1.00000   85952066      699050.67
      819199 1.00000   85952092     1048576.00
      831487 1.00000   85952103     1398101.33
      851967 1.00000   85952128     2097152.00
      856063 1.00000   85952133     2796202.67
      880639 1.00000   85952144     4194304.00
      884735 1.00000   85952151     5592405.33
      884735 1.00000   85952151     8388608.00
      888831 1.00000   85952159    11184810.67
      888831 1.00000   85952159    16777216.00
      888831 1.00000   85952159    22369621.33
      888831 1.00000   85952159    33554432.00
      892927 1.00000   85952161    44739242.67
      892927 1.00000   85952161            inf
#[Mean       =     18564.86, StdDeviation   =     23434.73]
#[Max        =       892927, Total count    =     85952161]
#[Buckets    =           30, SubBuckets     =         3968]


Throughput: 515.7 MB/s
            "#;

        let throughput = Some(515.7);

        assert_eq!(extract_throughput(some_log), throughput);
    }

    #[test]
    fn test_update_json() {
        let old_json = r#"[{"a":1},{"a":2}]"#;
        let to_be_added_json = r#"{"a":3}"#;
        let final_json = r#"[{"a":1},{"a":2},{"a":3}]"#;

        assert_eq!(update_json(old_json, to_be_added_json), final_json);
    }
}
