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

pub struct Limiter {
    client: Client,
    key: Vec<u8>,
}

impl Limiter {
    pub fn new<K: Into<Vec<u8>>>(client: Client, key: K) -> Self {
        Limiter {
            client,
            key: key.into(),
        }
    }

    pub async fn set_max_execute_times(&mut self, max_execute_times: i64) -> Result<bool> {
        let option = SetOption::default();
        let result = self
            .client
            .set(self.key.clone(), max_execute_times.to_string(), option)
            .await?;
        Ok(result)
    }

    pub async fn still_valid_to_execute(&mut self) -> Result<bool> {
        let result = self.client.decr(self.key.clone()).await?;
        Ok(result >= 0)
    }

    pub async fn remaining_execute_times(&mut self) -> Result<i64> {
        let result = self.client.get(self.key.clone()).await?;
        let result = Self::parse(result)?;
        Ok(result.max(0))
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
async fn test_limiter() -> Result<()> {
    let client = crate::client().await?;
    let mut limiter = Limiter::new(client, "wrong_password_limiter");

    let got = limiter.set_max_execute_times(3).await?;
    assert!(got);
    let got = limiter.still_valid_to_execute().await?;
    assert!(got);
    let got = limiter.remaining_execute_times().await?;
    assert_eq!(2, got);
    let got = limiter.still_valid_to_execute().await?;
    assert!(got);
    let got = limiter.remaining_execute_times().await?;
    assert_eq!(1, got);
    let got = limiter.still_valid_to_execute().await?;
    assert!(got);
    let got = limiter.remaining_execute_times().await?;
    assert_eq!(0, got);
    let got = limiter.still_valid_to_execute().await?;
    assert!(!got);
    let got = limiter.remaining_execute_times().await?;
    assert_eq!(0, got);
    let got = limiter.still_valid_to_execute().await?;
    assert!(!got);
    let got = limiter.remaining_execute_times().await?;
    assert_eq!(0, got);

    Ok(())
}
