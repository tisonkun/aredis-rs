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

struct Cache {
    client: Client,
}

impl Cache {
    pub fn new(client: Client) -> Self {
        Cache { client }
    }

    pub async fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.client.set(key, value, SetOption::default()).await?;
        Ok(())
    }

    pub async fn get(&mut self, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        Ok(self.client.get(key).await?)
    }

    pub async fn update(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<Option<Vec<u8>>> {
        Ok(self
            .client
            .get_set(key, value, SetOption::default())
            .await?)
    }
}

#[tokio::test]
async fn test_cache() -> Result<()> {
    let client = crate::client().await?;
    let mut cache = Cache::new(client);

    let key = "greeting-page".as_bytes();
    let first = "<html><p>hello world</p></html>".as_bytes();
    let second = "<html><p>good morning</p></html>".as_bytes();

    let got = cache.get(key.to_vec()).await?;
    assert!(got.is_none());

    cache.set(key.to_vec(), first.to_vec()).await?;

    let got = cache.get(key.to_vec()).await?;
    assert!(got.is_some());
    assert_eq!(got.unwrap(), first.to_vec());

    let got = cache.update(key.to_vec(), second.to_vec()).await?;
    assert!(got.is_some());
    assert_eq!(got.unwrap(), first.to_vec());

    let got = cache.get(key.to_vec()).await?;
    assert!(got.is_some());
    assert_eq!(got.unwrap(), second.to_vec());

    Ok(())
}
