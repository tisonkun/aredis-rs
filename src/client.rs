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

use crate::{
    command::{Del, FlushAll, Get, MGet, MSet, MSetNx, Ping, SetOption},
    Connection, Error, Model, Result,
};

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

    pub async fn get(&mut self, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        self.connection.send(Get::new(key)).await?;
        match self.connection.recv().await? {
            Some(Model::String(result)) => Ok(Some(result)),
            Some(Model::Nil) => Ok(None),
            model => match_failure(model),
        }
    }

    pub async fn set(&mut self, key: Vec<u8>, value: Vec<u8>, option: SetOption) -> Result<bool> {
        let set = option.build(key, value, false);
        self.connection.send(set).await?;
        match self.connection.recv().await? {
            Some(Model::Status(status)) if status.eq_ignore_ascii_case("OK") => Ok(true),
            Some(Model::Nil) => Ok(false),
            model => match_failure(model),
        }
    }

    pub async fn get_set(
        &mut self,
        key: Vec<u8>,
        value: Vec<u8>,
        option: SetOption,
    ) -> Result<Option<Vec<u8>>> {
        let set = option.build(key, value, true);
        self.connection.send(set).await?;
        match self.connection.recv().await? {
            Some(Model::Nil) => Ok(None),
            Some(Model::String(result)) => Ok(Some(result)),
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

    pub async fn del(&mut self, keys: Vec<Vec<u8>>) -> Result<i64> {
        self.connection.send(Del::new(keys)).await?;
        match self.connection.recv().await? {
            Some(Model::Integer(result)) => Ok(result),
            model => match_failure(model),
        }
    }

    pub async fn mset(&mut self, kvs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<()> {
        self.connection.send(MSet::new(kvs)).await?;
        match self.connection.recv().await? {
            Some(Model::Status(status)) if status.eq_ignore_ascii_case("OK") => Ok(()),
            model => match_failure(model),
        }
    }

    pub async fn msetnx(&mut self, kvs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<bool> {
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

    pub async fn mget(&mut self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>> {
        self.connection.send(MGet::new(keys)).await?;
        match self.connection.recv().await? {
            Some(Model::Array(models)) => {
                let mut result = vec![];
                for model in models.into_iter() {
                    match model {
                        Model::Nil => result.push(None),
                        Model::String(v) => result.push(Some(v)),
                        _ => return match_failure(Some(model)),
                    }
                }
                Ok(result)
            }
            model => match_failure(model),
        }
    }
}

fn match_failure<T>(model: Option<Model>) -> Result<T> {
    match model {
        Some(Model::Error(e)) => Err(Error::internal(e)),
        model => Err(Error::internal(format!("unreachable model: {:?}", model))),
    }
}
