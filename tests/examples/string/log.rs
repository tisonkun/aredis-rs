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
use aredis::Client;

use crate::Utf8String;

struct Log<K: Into<Vec<u8>> + Copy> {
    client: Client,
    key: K,
}

impl<K: Into<Vec<u8>> + Copy> Log<K> {
    const LOG_SEPARATOR: u8 = b'\n';

    pub fn new(client: Client, key: K) -> Self {
        Log { client, key }
    }

    pub async fn add<In>(&mut self, log: In) -> Result<()>
    where
        In: Into<Vec<u8>>,
    {
        let mut log = log.into();
        let mut sep = vec![Self::LOG_SEPARATOR];
        log.append(&mut sep);
        self.client.append(self.key.into(), log).await?;
        Ok(())
    }

    pub async fn get_all<Out>(&mut self) -> Result<Vec<Out>>
    where
        Out: From<Vec<u8>>,
    {
        match self.client.get::<K, Vec<u8>>(self.key).await? {
            None => Ok(vec![]),
            Some(logs) => {
                let mut result = vec![];
                for log in logs.split(|sep| *sep == Self::LOG_SEPARATOR) {
                    result.push(log.to_vec().into());
                }
                result.pop(); // drop the tailing empty string
                Ok(result)
            }
        }
    }
}

#[tokio::test]
async fn test_log() -> Result<()> {
    let client = crate::client().await?;
    let mut log = Log::new(client, "06 Jul");

    let log_0 = "17:40:49.611 # Server started, Redis version 3.1.999";
    let log_1 = "17:40:49.627 * DB loaded from disk: 0.016 seconds";
    let log_2 = "17:40:49.627 * The server is now ready to accept connections on port 6379";
    let log_3 = "18:29:20.009 * DB saved on disk";
    log.add(log_0).await?;
    log.add(log_1).await?;
    log.add(log_2).await?;
    log.add(log_3).await?;

    let logs: Vec<Utf8String> = log.get_all().await?;
    assert_eq!(logs.len(), 4);
    assert_eq!(logs[0], log_0.into());
    assert_eq!(logs[1], log_1.into());
    assert_eq!(logs[2], log_2.into());
    assert_eq!(logs[3], log_3.into());

    Ok(())
}
