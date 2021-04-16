import Head from "next/head";
import styles from "../styles/Home.module.css";

import BenchmarkChart from "../components/Benchmark.js";

// FIXME
import datas from "../data.json";

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

console.log(mapCombined);

export default function Home() {
  const charList = (
    Object.values(mapCombined).map((bench) =>
      <BenchmarkChart
        throughputs={bench.throughputs}
        commitList={bench.commits}
      />
    )
  );
  return (
    <div>
      {charList}
    </div>
  );
}
