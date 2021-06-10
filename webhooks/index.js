const express = require("express");
const { spawnSync } = require("child_process");
const app = express();

app.use(express.json());

app.post("/payload", function (req, res) {
  if (req.body.action === "completed") {
    const commit_hash = req.body.check_suite.head_commit.id;
    const child = spawnSync("./target/release/tremor-benchmark", [
      "./data/data.json",
      "./data/recent.json",
      commit_hash,
    ], { encoding: 'utf8' });

    if (child.error) {
      console.log("ERROR: ", child.error);
    }
    console.log("stdout: ", child.stdout);
    console.log("stderr: ", child.stderr);
    console.log("exit code: ", child.status);
  }
});

app.listen(3000);
