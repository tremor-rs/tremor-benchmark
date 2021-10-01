table! {
    benchmarks (id) {
        id -> Text,
        created_at -> Date,
        commit_hash -> Text,
        bench_name -> Text,
        mpbs -> Float,
        eps -> Float,
        hist -> Text,
    }
}
