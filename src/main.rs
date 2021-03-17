//! A helper tool for benchmarking tremor
//!
//! The purpose of this crate is threefold:
//! - Run the tremor benchmarks
//! - Store them in a structured format
//! - Create graphs based on historical data

#![warn(
    clippy::pedantic,
    clippy::cargo,
    clippy::perf,
    clippy::complexity,
    clippy::nursery,
    clippy::style,
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    box_pointers,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    invalid_html_tags,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_crate_level_docs,
    missing_debug_implementations,
    missing_doc_code_examples,
    missing_docs,
    non_ascii_idents,
    pointer_structural_match,
    private_doc_tests,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unaligned_references,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences,
    missing_docs,
    missing_doc_code_examples,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use clap::{crate_authors, crate_version, Clap};
use color_eyre::eyre::Result;

use std::path::PathBuf;
use std::process::Command;

/// This is utility used to run tremor benchmarks, store them in a structured format and generate
/// beautiful graphs based on historical data
#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    /// Path to the Tremor project
    root: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opts: Opts = Opts::parse();

    // FIXME Currently tremor's bench command doens't allow to run benchmarks to be run from other than
    // the project's root directory. See https://github.com/tremor-rs/tremor-runtime/issues/842

    // Change the current directory to tremor's root directory
    std::env::set_current_dir(&opts.root)?;

    // TODO Don't spawn process like this use the upstream code instead, it would be easier to
    // integrate in the future
    let output = Command::new("./target/release/tremor")
        .args(&["test", "bench", "./tremor-cli/tests/bench"])
        .output();

    dbg!(output?);

    Ok(())
}
