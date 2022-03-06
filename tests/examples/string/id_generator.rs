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

use anyhow::Result;
use aredis::{command::SetOption, Client};

pub struct IdGenerator {
    client: Client,
    key: Vec<u8>,
}

impl IdGenerator {
    pub fn new<K: Into<Vec<u8>>>(client: Client, key: K) -> Self {
        IdGenerator {
            client,
            key: key.into(),
        }
    }

    pub async fn produce(&mut self) -> Result<i64> {
        let result = self.client.incr(self.key.clone()).await?;
        Ok(result)
    }

    pub async fn reserve(&mut self, n: i64) -> Result<bool> {
        let option = SetOption::default().nx();
        let result = self
            .client
            .set(self.key.clone(), n.to_string(), option)
            .await?;
        Ok(result)
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_id_generator() -> Result<()> {
    let client = crate::client().await?;
    let mut id_generator = IdGenerator::new(client, "user::id");

    let got = id_generator.reserve(1000000).await?;
    assert!(got);

    let got = id_generator.produce().await?;
    assert_eq!(1000001, got);

    let got = id_generator.produce().await?;
    assert_eq!(1000002, got);

    let got = id_generator.produce().await?;
    assert_eq!(1000003, got);

    let got = id_generator.reserve(1000).await?;
    assert!(!got);

    Ok(())
}
