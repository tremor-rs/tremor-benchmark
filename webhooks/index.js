const express = require("express");
const app = express();

app.use(express.json());

app.post("/payload", function (req, res) {
  if (req.body.action === "completed") {
    const commit_hash = req.body.check_suite.head_commit.id;
    console.log(commit_hash);
  }
});

app.listen(3000);

