import Head from "next/head";
import BenchmarkChart from "../components/Benchmark.js";
import { useEffect, useState } from "react";

export default function Home() {
  const [datas, setDatas] = useState([]);
  useEffect(() => {
    fetch(
      "https://raw.githubusercontent.com/tremor-rs/tremor-benchmark/data/data.json"
    )
      .then((response) => {
        return response.json();
      })
      .then((result) => {
        setDatas(result);
      });
  }, []);
  let mapT = new Map();
  let mapC = new Map();

  function addValueToKey(map, key, value) {
    map[key] = map[key] || [];
    map[key].push(value);
  }

  function convert(jsonString) {
    let parsed_json = jsonString;
    let temporary_commit_array;
    parsed_json.forEach(function (entry) {
      entry.benchmarks.forEach(function (singleBenchEntry) {
        addValueToKey(mapT, singleBenchEntry.name, singleBenchEntry.throughput);
        addValueToKey(mapC, singleBenchEntry.name, entry.commit_hash);
      });
    });
  }

  convert(datas);
  let mapCombined = {};

  Object.keys(mapT).forEach(function (key) {
    let throughputs = mapT[key];
    let commits = mapC[key];
    mapCombined[key] = { throughputs, commits };
  });

  const charList = Object.keys(mapCombined).map((benchKey) => (
    <BenchmarkChart
      throughputs={mapCombined[benchKey].throughputs.slice(-20)}
      commitList={mapCombined[benchKey].commits.slice(-20)}
      title={benchKey}
    />
  ));
  return (
    <div>
      <Head>
        <title>Tremor Benchmarks</title>
      </Head>
      <main className="container mx-auto lg:px-80 px-4">
        <h1 className="text-center font-bold lg:text-5xl text-3xl pb-8 pt-6">
          Tremor Benchmarks
        </h1>
        <div>{charList}</div>
      </main>
    </div>
  );
}
