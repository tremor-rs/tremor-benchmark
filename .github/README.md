<h1 align="center">Tremor Benchmark</h1>
<div align="center">
 <strong>
   Continuous Benchmarking Infrastructure for Tremor
 </strong>
</div>

<div align="center">
  <h3>
    <a href="./CONTRIBUTING.md">
      Contributing
    </a>
    <span> | </span>
    <a href="https://discord.gg/Wjqu5H9rhQ">
      Chat
    </a>
  </h3>
</div>

This is the repository for the Continuous Benchmarking Infrastructure for Tremor
inspired by [Deno's Continuous Benchmarks](https://deno.land/benchmarks)

A good Continuous Benchmarking System in our opinion requires three things:

- The benchmarks themselves along with _something_ to run those benchmarks
- _Someplace_ to store the benchmark data
- _Something_ to view the stored benchmark data

The
[tremor-cli](https://github.com/tremor-rs/tremor-runtime/tree/main/tremor-cli)
is used to run the benchmarks and a `report.json` is generated which is then
converted to a JSON file which can be stored and is easy to parse for historical
report generation. There are two files stored as JSON in the data/ folder
`data.json` and `recent.json`. `data.json` contains the data of all the
benchmarks from the start and `recent.json` contains the data of the last
benchmark run. The website/ folder contains the code of the website which
renders the charts for the benchmarks

## Contributing

Want to join us? Check out our
[The "Contributing" section of the guide][contributing] and take a look at some
of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

#### Conduct

This project adheres to
[The Tremor Code of Conduct](https://github.com/tremor-rs/tremor-runtime/blob/main/CODE_OF_CONDUCT.md).
This describes the minimum behavior expected from all contributors.

## License

Licensed under

- Apache License, Version 2.0 ([LICENSE](LICENSE)
  https://www.apache.org/licenses/LICENSE-2.0)

<!-- not sure what this syntax is -->

[contributing]: https://github.com/humancalico/tremor-benchmark/blob/main/.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/humancalico/tremor-benchmark/labels/good%20first%20issue
[help-wanted]: https://github.com/humancalico/tremor-benchmark/labels/help%20wanted
