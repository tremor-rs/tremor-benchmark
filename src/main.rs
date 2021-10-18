// Copyright 2020-2021, The Tremor Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate diesel;

mod error;
mod model;
pub(crate) mod schema;
mod util;

use crate::error::Error;
use crate::schema::benchmarks;
use crate::schema::benchmarks::dsl::*;
use crate::util::convert_into_relevant_data;
use async_std::channel::{bounded, Sender};
use async_std::task;
use clap::{crate_authors, crate_version, Clap};
use color_eyre::eyre::Result;
use diesel::prelude::*;
use diesel::{Connection, SqliteConnection};
use model::Benchmark;
use serde_json::Value;

use async_std::process::Command;
use async_std::sync::Arc;
use std::env;

use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};

use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clap, Debug, Clone)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    /// The key to validate github with
    key: Option<String>,
}

async fn get_report(hash: &str) -> Result<Vec<Benchmark>> {
    // calculate short commit hash
    let short_commit_hash = &hash[..6];

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
            &format!("commithash={}", hash),
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
    convert_into_relevant_data(serde_json::from_slice(&r.stdout)?, hash)
}

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn run(
    opts: Arc<Opts>,
    tx: Sender<String>,
    req: Request<Body>,
) -> Result<Response<Body>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/bench") => {
            // FIXME: this is terrible
            let connection = establish_connection();
            let mut res: Vec<Benchmark> = benchmarks
                .order(created_at.desc())
                .limit(100)
                .load(&connection)?;
            res.reverse();
            let res = serde_json::to_string(&res)?;
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(res))
                .map_err(|_| Error::Other)
        }
        // Simply echo the body back to the client.
        (&Method::POST, "/bench") => {
            //
            let sig = req
                .headers()
                .get("X-Hub-Signature-256")
                .and_then(|v| v.to_str().ok())
                .map(ToString::to_string);

            let event = req
                .headers()
                .get("X-GitHub-Event")
                .and_then(|v| v.to_str().ok())
                .map(ToString::to_string)
                .ok_or_else(|| Error::BadRequest("missing X-GitHub-Event header".into()))?;

            if event != "push" {
                return Err(Error::BadRequest(format!(
                    "Only runs on `push` but got {}",
                    event
                )));
            };

            let body = hyper::body::to_bytes(req.into_body()).await?;

            if let Some(key) = &opts.key {
                let mut mac = HmacSha256::new_from_slice(key.as_bytes())?;
                mac.update(&body);
                let result = format!(
                    "sha256={}",
                    base16::encode_lower(&mac.finalize().into_bytes())
                );
                if sig != Some(result) {
                    //FIXME: Error
                    let mut error = Response::new(Body::from("bad hmac"));
                    *error.status_mut() = StatusCode::FORBIDDEN;
                    return Ok(error);
                };
            };

            let body = serde_json::from_slice::<Value>(&body)?;

            let ghref = body
                .get("ref")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::BadRequest("`ref` is missing".into()))?
                .to_string();

            if ghref != "main" {
                return Ok(Response::new(Body::from(format!(
                    r#"{{"branch": "{}"}}"#,
                    ghref
                ))));
            }

            let hash = body
                .get("after")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::BadRequest("`after` is missing".into()))?
                .to_string();

            tx.send(hash.clone()).await?;

            Ok(Response::new(Body::from(format!(
                r#"{{"hash": "{}"}}"#,
                hash
            ))))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut error = Response::new(Body::from("not found"));
            *error.status_mut() = StatusCode::NOT_FOUND;
            Ok(error)
        }
    }
}

fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();

    let (tx, rx) = bounded::<String>(64);

    task::spawn(async move {
        let connection = establish_connection();
        while let Ok(hash) = rx.recv().await {
            match get_report(&hash).await {
                Err(e) => eprint!("Report Error {}", e),
                Ok(r) => {
                    println!("data: {:?}", r);
                    for b in r {
                        diesel::insert_into(benchmarks::table)
                            .values(&b.as_new())
                            .execute(&connection)
                            .expect("Error saving new post");
                    }
                }
            };
        }
    });
    let opts: Opts = Opts::parse();

    let addr = ([127, 0, 0, 1], 3001).into();

    let service = make_service_fn(move |_| {
        let o = Arc::new(opts.clone());
        let tx = tx.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let o = o.clone();
                let tx = tx.clone();

                async move {
                    match run(o, tx, req).await {
                        Ok(r) => Ok(r),
                        Err(Error::BadRequest(e)) => {
                            let mut error = Response::new(Body::from(e));
                            *error.status_mut() = StatusCode::BAD_REQUEST;
                            Ok(error)
                        }
                        Err(Error::Hyper(e)) => Err(e),
                        Err(e) => {
                            let mut error = Response::new(Body::from(format!("Error: {:?}", e)));
                            *error.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            Ok(error)
                        }
                    }
                }
            }))
        }
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
