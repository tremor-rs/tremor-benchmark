-- Your SQL goes here
CREATE TABLE benchmarks (
    id VARCHAR NOT NULL PRIMARY KEY,
    created_at DATE NOT NULL,
    commit_hash CHAR(40)  NOT NULL,
    bench_name VARCHAR  NOT NULL,
    mpbs FLOAT8 NOT NULL,
    eps FLOAT8 NOT NULL,
    hist TEXT NOT NULL
);