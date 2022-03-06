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

use anyhow::{Error, Result};
use aredis::{command::SetOption, Client};

pub struct Counter {
    client: Client,
    key: Vec<u8>,
}

impl Counter {
    pub fn new<K: Into<Vec<u8>>>(client: Client, key: K) -> Self {
        Counter {
            client,
            key: key.into(),
        }
    }

    pub async fn increase(&mut self, n: Option<i64>) -> Result<i64> {
        let result = self
            .client
            .incr_by(self.key.clone(), n.unwrap_or(1))
            .await?;
        Ok(result)
    }

    pub async fn decrease(&mut self, n: Option<i64>) -> Result<i64> {
        let result = self
            .client
            .decr_by(self.key.clone(), n.unwrap_or(1))
            .await?;
        Ok(result)
    }

    pub async fn get(&mut self) -> Result<i64> {
        let result = self.client.get(self.key.clone()).await?;
        Self::parse(result)
    }

    pub async fn reset(&mut self) -> Result<i64> {
        let option = SetOption::default();
        let result = self
            .client
            .get_set(self.key.clone(), 0.to_string(), option)
            .await?;
        Self::parse(result)
    }

    fn parse(bytes: Option<Vec<u8>>) -> Result<i64> {
        match bytes {
            None => Ok(0),
            Some(i) => atoi::atoi(i.as_slice()).ok_or_else(|| Error::msg("malformed int")),
        }
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_counter() -> Result<()> {
    let client = crate::client().await?;
    let mut counter = Counter::new(client, "counter::page_view");

    let got = counter.increase(None).await?;
    assert_eq!(1, got);
    let got = counter.increase(None).await?;
    assert_eq!(2, got);
    let got = counter.increase(Some(10)).await?;
    assert_eq!(12, got);
    let got = counter.decrease(None).await?;
    assert_eq!(11, got);
    let got = counter.decrease(Some(5)).await?;
    assert_eq!(6, got);
    let got = counter.reset().await?;
    assert_eq!(6, got);
    let got = counter.get().await?;
    assert_eq!(0, got);

    Ok(())
}
