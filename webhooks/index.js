// TODO RiiR
const { spawnSync } = require("child_process");
const crypto = require("crypto");

const express = require("express");
require('dotenv').config();

const app = express();

app.use(express.json({
  verify: (req, res, buf, encoding) => {
    if (buf && buf.length) {
      req.rawBody = buf.toString(encoding || 'utf8');
    }
  },
}));

const sigHeaderName = 'X-Hub-Signature-256'
const sigHashAlg = 'sha256';

function verify(req, res, next) {
  if (!req.rawBody) {
    return next('Request body empty')
  }
  const sig = Buffer.from(req.get(sigHeaderName) || '', 'utf8')
  const hmac = crypto.createHmac(sigHashAlg, process.env.SECRET)
  const digest = Buffer.from(sigHashAlg + '=' + hmac.update(req.rawBody).digest('hex'), 'utf8')
  if (sig.length !== digest.length || !crypto.timingSafeEqual(digest, sig)) {
    return next(`Request body digest (${digest}) did not match ${sigHeaderName} (${sig})`)
  }

  next()
}

app.post("/payload", verify, function (req, res) {
  if (req.body.action === "completed") {
    const commit_hash = req.body.check_suite.head_commit.id;
    // respond to the post request
    res.status(200).send(`Received a request for the commit ${commit_hash} and request body was signed`);
    // start the benchmark pipeline
    const child = spawnSync("./target/release/tremor-benchmark", [
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

app.use((err, req, res, next) => {
  if (err) console.error(err)
  res.status(403).send('Request body was not signed or verification failed')
})

// should this be hardcoded?
const port = 9247;
const address = '0.0.0.0';
app.listen(port, address, () => console.log(`Listening on port ${address}:${port}`));
