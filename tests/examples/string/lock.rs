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

struct Lock {
    client: Client,
    key: Vec<u8>,
}

impl Lock {
    const VALUE_ON_LOCK: &'static [u8] = b"locking";

    pub fn new(client: Client, key: Vec<u8>) -> Self {
        Self { client, key }
    }

    pub async fn acquire(&mut self) -> Result<bool> {
        let key = self.key.clone();
        let value = Self::VALUE_ON_LOCK.to_vec();
        let option = SetOption::default().nx();
        let result = self.client.set(key, value, option).await?;
        Ok(result)
    }

    pub async fn release(&mut self) -> Result<bool> {
        let key = self.key.clone();
        let result = self.client.del(vec![key]).await?;
        Ok(result == 1)
    }
}

#[tokio::test]
async fn test_lock() -> Result<()> {
    let client = crate::client().await?;
    let mut lock = Lock::new(client, "test-lock".as_bytes().to_vec());

    let got = lock.acquire().await?;
    assert!(got);

    let got = lock.acquire().await?;
    assert!(!got);

    let got = lock.release().await?;
    assert!(got);

    let got = lock.acquire().await?;
    assert!(got);
    Ok(())
}
