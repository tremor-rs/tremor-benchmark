use std::fmt::Display;

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
use hmac::crypto_mac::InvalidKeyLength;

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Self::Hyper(e)
    }
}
impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Self::Text(e.to_string())
    }
}
impl From<InvalidKeyLength> for Error {
    fn from(_: InvalidKeyLength) -> Self {
        Self::Other
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::BadRequest(format!("Invalid JSON: {}", e))
    }
}
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::Other
    }
}
impl<T> From<async_std::channel::SendError<T>> for Error {
    fn from(_: async_std::channel::SendError<T>) -> Self {
        Self::Other
    }
}
impl From<diesel::result::Error> for Error {
    fn from(_: diesel::result::Error) -> Self {
        Self::Other
    }
}

#[derive(Debug)]
pub enum Error {
    Other,
    Text(String),
    Hyper(hyper::Error),
    BadRequest(String),
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
