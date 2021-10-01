use super::schema::benchmarks;
use serde::Serialize;

#[derive(Serialize, Queryable, Debug)]
pub struct Benchmark {
    pub id: String,
    pub created_at: String,
    pub commit_hash: String,
    pub bench_name: String,
    pub mpbs: f32,
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
            mpbs: self.mpbs,
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
    pub mpbs: f32,
    pub eps: f32,
    pub hist: &'a str,
}
