use async_std::channel::{Sender, bounded};
use async_std::task;
use clap::{crate_authors, crate_version, Clap};
use color_eyre::eyre::Result;
use serde_json::Value;
use tremor_benchmark::{convert_into_relevant_data, Data};

use async_std::process::Command;
use async_std::sync::Arc;
use std::env;

use tremor_benchmark::deserialize;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clap, Debug, Clone)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    /// The key to validate github with
    key: Option<String>,
}

async fn get_report(commit_hash: &str) -> Result<Data> {
    // calculate short commit hash
    let short_commit_hash = &commit_hash[..6];

    let tag = format!("tremor-benchmark:{}", short_commit_hash);

    // add the commit hash as tag
    Command::new("docker")
        .args(&[
            "build",
            "-t",
            &tag,
            "-f",
            "Dockerfile.bench",
            "--build-arg",
            &format!("commithash={}", commit_hash),
            "docker",
        ])
        .output()
        .await?;

    // run benchmarks inside docker image and store it in a report.json
    let r = Command::new("docker").args(["run", &tag]).output().await?;
    Command::new("docker")
        .args(&["image", "rm", &tag])
        .output()
        .await?;
    convert_into_relevant_data(deserialize(&r.stdout)?, commit_hash)
}



/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn run(opts: Arc<Opts>, tx: Sender<String>, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Try POSTing data to /echo such as: `curl localhost:3000/bench -XPOST -d '<commit hash>'`",
        ))),

        // Simply echo the body back to the client.
        (&Method::POST, "/bench") => {
            //
            let sig = req.headers().get("X-Hub-Signature-256").and_then(|v| v.to_str().ok()).map(ToString::to_string);
            let body = hyper::body::to_bytes(req.into_body()).await?;

            if let Some(key) = &opts.key {
                let mut mac = HmacSha256::new_from_slice(key.as_bytes())    .expect("HMAC failed to create");
                mac.update(&body);
                let result = base16::encode_lower( &mac.finalize().into_bytes());
                if  sig != Some(result) {
                    //FIXME: Error
                    let mut error = Response::new(Body::from("bad hmac"));
                    *error.status_mut() = StatusCode::FORBIDDEN;
                    return Ok(error)
                };
            };

            let body = if let Ok(b) = serde_json::from_slice::<Value>(&body) {
                b
            } else {
                let mut error = Response::new(Body::from("invalid payload"));
                    *error.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(error) 
            };

            if Some("push") != 
                body.get("action").and_then(Value::as_str) {
                    //FIXME: Error
                    //FIXME: Error
                    let mut error = Response::new(Body::from("action missing or not `push`"));
                    *error.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(error)
            };

            let hash = if let Some(s) = body.get("after").and_then(Value::as_str) {
                s.to_string()
            } else {
                //FIXME: Error
                let mut error = Response::new(Body::from("after missing"));
                *error.status_mut() = StatusCode::FORBIDDEN;
                return Ok(error)
            };


            tx.send(hash.clone()).await.unwrap();

            Ok(Response::new(Body::from(format!(r#"{{"hash": "{}"}}"#, hash))))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let (tx, rx) = bounded::<String>(64);

    task::spawn(async move {
        while let Ok(hash) = rx.recv().await {
            match  get_report(&hash).await {
                Err(e) => eprint!("Report Error {}", e),
                Ok(r) => println!("data: {:?}", r)
            };
        }
    });
    let opts: Opts = Opts::parse();

    let addr = ([127, 0, 0, 1], 3000).into();

    let service = make_service_fn(move |_| {
        let o = Arc::new(opts.clone());
        let tx = tx.clone();
        async move { Ok::<_, hyper::Error>(service_fn(move |req| run(o.clone(), tx.clone(), req))) }
    });

    let server = Server::bind(&addr).serve(service);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

// fn main() -> Result<()> {
//     color_eyre::install()?;
//     // TODO check for dependencies like docker, docker-compose and git
//     // TODO should be able to use other container runtimes other than docker compose like podman

//     let opts: Opts = Opts::parse();
//     let report = get_report(&opts.commit_hash)?;

//     // parse the report into data.json and recent.json
//     fs::write(
//         &opts.recent_file,
//         serialize(&convert_into_relevant_data(report, &opts.commit_hash)?)?,
//     )?;

//     fs::write(
//         &opts.data_file,
//         update_json(
//             &fs::read_to_string(&opts.data_file)?,
//             &fs::read_to_string(opts.recent_file)?,
//         ),
//     )?;

//     Ok(())
// }
