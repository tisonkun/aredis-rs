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

use crate::Utf8String;

struct Cache {
    client: Client,
}

impl Cache {
    pub fn new(client: Client) -> Self {
        Cache { client }
    }

    pub async fn set<In0, In1>(&mut self, key: In0, value: In1) -> Result<()>
    where
        In0: Into<Vec<u8>>,
        In1: Into<Vec<u8>>,
    {
        self.client.set(key, value, SetOption::default()).await?;
        Ok(())
    }

    pub async fn get<In, Out>(&mut self, key: In) -> Result<Option<Out>>
    where
        In: Into<Vec<u8>>,
        Out: From<Vec<u8>>,
    {
        Ok(self.client.get(key).await?)
    }

    pub async fn update<In0, In1, Out>(&mut self, key: In0, value: In1) -> Result<Option<Out>>
    where
        In0: Into<Vec<u8>>,
        In1: Into<Vec<u8>>,
        Out: From<Vec<u8>>,
    {
        let option = SetOption::default();
        Ok(self.client.get_set(key, value, option).await?)
    }
}

#[tokio::test]
async fn test_cache() -> Result<()> {
    let client = crate::client().await?;
    let mut cache = Cache::new(client);

    let key = "greeting-page";
    let first = "<html><p>hello world</p></html>";
    let second = "<html><p>good morning</p></html>";

    let got: Option<Utf8String> = cache.get(key).await?;
    assert!(got.is_none());

    cache.set(key, first).await?;

    let got: Option<Utf8String> = cache.get(key).await?;
    assert!(got.is_some());
    assert_eq!(got.unwrap(), first.into());

    let got: Option<Utf8String> = cache.update(key, second).await?;
    assert!(got.is_some());
    assert_eq!(got.unwrap(), first.into());

    let got: Option<Utf8String> = cache.get(key).await?;
    assert!(got.is_some());
    assert_eq!(got.unwrap(), second.into());

    Ok(())
}
