table! {
    benchmarks (id) {
        id -> Text,
        created_at -> Date,
        commit_hash -> Text,
        bench_name -> Text,
        mbps -> Float,
        eps -> Float,
        hist -> Text,
    }
}
