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

mod examples;

pub async fn client() -> anyhow::Result<Client> {
    let host = option_env!("REDIS_HOST").unwrap_or_else(|| "localhost");
    let port = option_env!("REDIS_PORT").unwrap_or_else(|| "6379");
    let mut client = Client::connect(format!("{}:{}", host, port)).await?;
    client.flush_all(true).await?;
    Ok(client)
}
