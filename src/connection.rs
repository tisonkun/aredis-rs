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

use std::io::Cursor;

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::{command::Command, model, model::Model, Error, Result};

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn send(&mut self, cmd: impl Command) -> Result<()> {
        let req = cmd.as_bytes();
        self.stream.write_all(req.as_slice()).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Option<Model>> {
        loop {
            if let Some(model) = self.parse()? {
                break Ok(Some(model));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                break if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err(Error::internal("connection reset by peer"))
                };
            }
        }
    }

    pub fn parse(&mut self) -> Result<Option<Model>> {
        let mut cursor = Cursor::new(&self.buffer[..]);
        match model::check(&mut cursor) {
            Ok(()) => {
                let len = cursor.position() as usize;
                cursor.set_position(0);
                let model = model::parse(&mut cursor)?;
                self.buffer.advance(len);
                Ok(Some(model))
            }
            Err(error) if error.code().is_incomplete() => Ok(None),
            Err(error) => Err(error),
        }
    }
}
