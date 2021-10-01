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

use super::schema::benchmarks;
use serde::Serialize;

#[derive(Serialize, Queryable, Debug)]
pub struct Benchmark {
    pub id: String,
    pub created_at: String,
    pub commit_hash: String,
    pub bench_name: String,
    pub mbps: f32,
    pub eps: f32,
    pub hist: String,
}

impl Benchmark {
    pub fn as_new(&self) -> NewBenchmark {
        NewBenchmark {
            id: &self.id,
            created_at: &self.created_at,
            commit_hash: &self.commit_hash,
            bench_name: &self.bench_name,
            mbps: self.mbps,
            eps: self.eps,
            hist: &self.hist,
        }
    }
}

#[derive(Insertable)]
#[table_name = "benchmarks"]
pub struct NewBenchmark<'a> {
    pub id: &'a str,
    pub created_at: &'a str,
    pub commit_hash: &'a str,
    pub bench_name: &'a str,
    pub mbps: f32,
    pub eps: f32,
    pub hist: &'a str,
}
