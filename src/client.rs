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

use tokio::net::{TcpStream, ToSocketAddrs};

use crate::{command::*, Connection, Error, Model, Result};

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let socket = TcpStream::connect(addr).await?;
        let connection = Connection::new(socket);
        Ok(Self { connection })
    }

    pub async fn ping(&mut self) -> Result<()> {
        self.connection.send(Ping).await?;
        match self.connection.recv().await? {
            Some(Model::Status(status)) if status.eq_ignore_ascii_case("PONG") => Ok(()),
            model => match_failure(model),
        }
    }

    pub async fn get<In, Out>(&mut self, key: In) -> Result<Option<Out>>
    where
        In: Into<Vec<u8>>,
        Out: From<Vec<u8>>,
    {
        self.connection.send(Get::new(key.into())).await?;
        match self.connection.recv().await? {
            Some(Model::String(result)) => Ok(Some(result.into())),
            Some(Model::Nil) => Ok(None),
            model => match_failure(model),
        }
    }

    pub async fn set<In0, In1>(&mut self, key: In0, value: In1, option: SetOption) -> Result<bool>
    where
        In0: Into<Vec<u8>>,
        In1: Into<Vec<u8>>,
    {
        let set = option.build(key.into(), value.into(), false);
        self.connection.send(set).await?;
        match self.connection.recv().await? {
            Some(Model::Status(status)) if status.eq_ignore_ascii_case("OK") => Ok(true),
            Some(Model::Nil) => Ok(false),
            model => match_failure(model),
        }
    }

    pub async fn get_set<In0, In1, Out>(
        &mut self,
        key: In0,
        value: In1,
        option: SetOption,
    ) -> Result<Option<Out>>
    where
        In0: Into<Vec<u8>>,
        In1: Into<Vec<u8>>,
        Out: From<Vec<u8>>,
    {
        let set = option.build(key.into(), value.into(), true);
        self.connection.send(set).await?;
        match self.connection.recv().await? {
            Some(Model::Nil) => Ok(None),
            Some(Model::String(result)) => Ok(Some(result.into())),
            model => match_failure(model),
        }
    }

    pub async fn flush_all(&mut self, sync: bool) -> Result<()> {
        self.connection.send(FlushAll::new(sync)).await?;
        match self.connection.recv().await? {
            Some(Model::Status(status)) if status.eq_ignore_ascii_case("OK") => Ok(()),
            model => match_failure(model),
        }
    }

    pub async fn del<In>(&mut self, keys: Vec<In>) -> Result<u64>
    where
        In: Into<Vec<u8>>,
    {
        let keys = keys.into_iter().map(|k| k.into()).collect();
        self.connection.send(Del::new(keys)).await?;
        match self.connection.recv().await? {
            Some(Model::Integer(result)) if result >= 0 => Ok(result as u64),
            model => match_failure(model),
        }
    }

    pub async fn exists<In>(&mut self, keys: Vec<In>) -> Result<u64>
    where
        In: Into<Vec<u8>>,
    {
        let keys = keys.into_iter().map(|k| k.into()).collect();
        self.connection.send(Exists::new(keys)).await?;
        match self.connection.recv().await? {
            Some(Model::Integer(result)) if result >= 0 => Ok(result as u64),
            model => match_failure(model),
        }
    }

    pub async fn mset<In0, In1>(&mut self, kvs: Vec<(In0, In1)>) -> Result<()>
    where
        In0: Into<Vec<u8>>,
        In1: Into<Vec<u8>>,
    {
        let kvs = kvs
            .into_iter()
            .map(|kv| (kv.0.into(), kv.1.into()))
            .collect();
        self.connection.send(MSet::new(kvs)).await?;
        match self.connection.recv().await? {
            Some(Model::Status(status)) if status.eq_ignore_ascii_case("OK") => Ok(()),
            model => match_failure(model),
        }
    }

    pub async fn msetnx<In0, In1>(&mut self, kvs: Vec<(In0, In1)>) -> Result<bool>
    where
        In0: Into<Vec<u8>>,
        In1: Into<Vec<u8>>,
    {
        let kvs = kvs
            .into_iter()
            .map(|kv| (kv.0.into(), kv.1.into()))
            .collect();
        self.connection.send(MSetNx::new(kvs)).await?;
        match self.connection.recv().await? {
            Some(Model::Integer(result)) => match result {
                0 => Ok(false),
                1 => Ok(true),
                _ => match_failure(Some(Model::Integer(result))),
            },
            model => match_failure(model),
        }
    }

    pub async fn mget<In, Out>(&mut self, keys: Vec<In>) -> Result<Vec<Option<Out>>>
    where
        In: Into<Vec<u8>>,
        Out: From<Vec<u8>>,
    {
        let keys = keys.into_iter().map(|k| k.into()).collect();
        self.connection.send(MGet::new(keys)).await?;
        match self.connection.recv().await? {
            Some(Model::Array(models)) => {
                let mut result = vec![];
                for model in models.into_iter() {
                    match model {
                        Model::Nil => result.push(None),
                        Model::String(v) => result.push(Some(v.into())),
                        _ => return match_failure(Some(model)),
                    }
                }
                Ok(result)
            }
            model => match_failure(model),
        }
    }

    pub async fn strlen<In>(&mut self, key: In) -> Result<u64>
    where
        In: Into<Vec<u8>>,
    {
        self.connection.send(Strlen::new(key.into())).await?;
        match self.connection.recv().await? {
            Some(Model::Integer(len)) if len >= 0 => Ok(len as u64),
            model => match_failure(model),
        }
    }

    pub async fn get_range<In, Out>(&mut self, key: In, start: i64, end: i64) -> Result<Out>
    where
        In: Into<Vec<u8>>,
        Out: From<Vec<u8>>,
    {
        self.connection
            .send(GetRange::new(key.into(), start, end))
            .await?;
        match self.connection.recv().await? {
            Some(Model::String(result)) => Ok(result.into()),
            model => match_failure(model),
        }
    }

    pub async fn set_range<In0, In1>(
        &mut self,
        key: In0,
        index: i64,
        substitute: In1,
    ) -> Result<u64>
    where
        In0: Into<Vec<u8>>,
        In1: Into<Vec<u8>>,
    {
        self.connection
            .send(SetRange::new(key.into(), index, substitute.into()))
            .await?;
        match self.connection.recv().await? {
            Some(Model::Integer(len)) if len >= 0 => Ok(len as u64),
            model => match_failure(model),
        }
    }

    pub async fn append<In0, In1>(&mut self, key: In0, suffix: In1) -> Result<u64>
    where
        In0: Into<Vec<u8>>,
        In1: Into<Vec<u8>>,
    {
        self.connection
            .send(Append::new(key.into(), suffix.into()))
            .await?;
        match self.connection.recv().await? {
            Some(Model::Integer(len)) if len >= 0 => Ok(len as u64),
            model => match_failure(model),
        }
    }

    pub async fn incr<In>(&mut self, key: In) -> Result<i64>
    where
        In: Into<Vec<u8>>,
    {
        self.connection.send(Incr::new(key.into())).await?;
        match self.connection.recv().await? {
            Some(Model::Integer(result)) => Ok(result),
            model => match_failure(model),
        }
    }

    pub async fn incr_by<In>(&mut self, key: In, increment: i64) -> Result<i64>
    where
        In: Into<Vec<u8>>,
    {
        self.connection
            .send(IncrBy::new(key.into(), increment))
            .await?;
        match self.connection.recv().await? {
            Some(Model::Integer(result)) => Ok(result),
            model => match_failure(model),
        }
    }

    pub async fn incr_by_float<In, Out>(&mut self, key: In, increment: f64) -> Result<f64>
    where
        In: Into<Vec<u8>>,
    {
        self.connection
            .send(IncrByFloat::new(key.into(), increment))
            .await?;
        match self.connection.recv().await? {
            Some(Model::String(result)) => {
                let result = String::from_utf8(result)?;
                let result = result.parse()?;
                Ok(result)
            }
            model => match_failure(model),
        }
    }

    pub async fn decr<In>(&mut self, key: In) -> Result<i64>
    where
        In: Into<Vec<u8>>,
    {
        self.connection.send(Decr::new(key.into())).await?;
        match self.connection.recv().await? {
            Some(Model::Integer(result)) => Ok(result),
            model => match_failure(model),
        }
    }

    pub async fn decr_by<In>(&mut self, key: In, decrement: i64) -> Result<i64>
    where
        In: Into<Vec<u8>>,
    {
        self.connection
            .send(DecrBy::new(key.into(), decrement))
            .await?;
        match self.connection.recv().await? {
            Some(Model::Integer(result)) => Ok(result),
            model => match_failure(model),
        }
    }
}

fn match_failure<T>(model: Option<Model>) -> Result<T> {
    match model {
        Some(Model::Error(e)) => Err(Error::Server(e)),
        model => Err(Error::Internal(format!("unreachable model: {:?}", model))),
    }
}
