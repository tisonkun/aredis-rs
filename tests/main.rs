// Copyright 2022 tison <wander4096@gmail.com>.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use aredis::Client;

mod commands;
mod examples;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct Utf8String(String);

impl From<Vec<u8>> for Utf8String {
    fn from(bytes: Vec<u8>) -> Self {
        let inner = String::from_utf8(bytes).unwrap();
        Self(inner)
    }
}

impl From<&'static str> for Utf8String {
    fn from(s: &'static str) -> Self {
        Self(s.to_string())
    }
}

pub async fn client() -> anyhow::Result<Client> {
    let host = option_env!("REDIS_HOST").unwrap_or_else(|| "localhost");
    let port = option_env!("REDIS_PORT").unwrap_or_else(|| "6379");
    let mut client = Client::connect(format!("{}:{}", host, port)).await?;
    client.flush_all(true).await?;
    Ok(client)
}
