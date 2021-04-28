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
  const charList = Object.keys(mapCombined).map((benchKey) => (
    <BenchmarkChart
      throughputs={mapCombined[benchKey].throughputs}
      commitList={mapCombined[benchKey].commits}
      title={benchKey}
    />
  ));
  return (
    <div className={styles.container}>
      <Head>
        <title>Tremor Benchmarks</title>
      </Head>
      <main className={styles.main}>
        <h1 className={styles.title}>Tremor Benchmarks</h1>
        <div>{charList}</div>
      </main>
    </div>
  );
}
