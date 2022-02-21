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
use aredis::command::SetOption;

#[tokio::test]
async fn test_strlen() -> Result<()> {
    let mut client = crate::client().await?;
    client.set("number", "10086", SetOption::default()).await?;
    let got = client.strlen("number").await?;
    assert_eq!(got, 5);
    client.set("empty", "", SetOption::default()).await?;
    let got = client.strlen("empty").await?;
    assert_eq!(got, 0);
    let got = client.strlen("nonexisting").await?;
    assert_eq!(got, 0);
    Ok(())
}
