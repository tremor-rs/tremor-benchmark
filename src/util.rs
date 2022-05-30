// Copyright 2020-2021, The Tremor Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use chrono::offset::Utc;
use color_eyre::eyre::Result;
use serde::Deserialize;

use crate::error::Error;

// TODO The name is horrible here. pls help
#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct SingleReport {
    description: String,
    elements: Element,
}

#[derive(Deserialize, Debug)]
struct Element {
    bench: Bench,
}

#[allow(dead_code)]
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
#[allow(dead_code)]
struct Evidence {
    #[serde(rename = "test: stdout")]
    stdout: String,
    #[serde(rename = "test: stderr")]
    stderr: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
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
#[allow(dead_code)]
struct Stats {
    command: Stat,
    all: Stat,
    integration: Stat,
    unit: Stat,
    bench: Stat,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Stat {
    pass: u16,
    fail: u16,
    skip: u16,
    assert: u16,
}

pub fn convert_into_relevant_data(
    whole_report: WholeReport,
    commit_hash: &str,
) -> Result<Vec<crate::model::Benchmark>> {
    let created_at = Utc::now().to_string();

    whole_report
        .reports
        .bench
        .into_iter()
        .map(|report| {
            // TODO add a check if benchmark has passed or failed
            let r = &report.elements.bench.evidence.stdout;
            let mbps = extract_throughput(r).ok_or(Error::Other("failed to get mbps"));
            let eps = extract_events(r).ok_or(Error::Other("failed to get eps"));
            let hist = extract_hist(r).ok_or(Error::Other("faild to get histogram"));

            if mbps.is_err() || eps.is_err() || hist.is_err() {
                dbg!(&report);
            }

            let mbps = mbps.unwrap_or_default();
            let eps = eps.unwrap_or_default();
            let hist = hist.unwrap_or_default().to_string();

            let bench_name = report.elements.bench.name;
            Ok(crate::model::Benchmark {
                id: format!("{}-{}-{}", commit_hash, &bench_name, &created_at),
                created_at: created_at.clone(),
                commit_hash: commit_hash.to_string(),
                bench_name,
                mbps,
                eps,
                hist,
            })
        })
        .collect()
}

fn extract_throughput(log_string: &str) -> Option<f32> {
    let start = log_string.find("Throughput   (data):")? + 21;
    let end = log_string.find("MB/s")?;

    log_string
        .get(start..end)
        .and_then(|throughput_string| throughput_string.trim().parse().ok())
}

fn extract_events(log_string: &str) -> Option<f32> {
    let start = log_string.find("Throughput (events):")? + 21;
    let end = log_string.find("k events/s")?;

    log_string
        .get(start..end)
        .and_then(|throughput_string| throughput_string.trim().parse().ok())
}

fn extract_hist(log_string: &str) -> Option<&str> {
    let end = log_string.find("\n\n\n")?;

    log_string.get(..end)
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;
    use pretty_assertions::assert_eq;

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


Throughput   (data): 58.7 MB/s
Throughput (events): 921.6k events/s
            "#;

        let hist = r#"
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
#[Buckets    =           30, SubBuckets     =         3968]"#;
        let throughput = Some(58.7);
        let events = Some(921.6);
        assert_eq!(extract_throughput(some_log), throughput);
        assert_eq!(extract_events(some_log), events);
        assert_eq!(extract_hist(some_log), Some(hist));
    }
}
