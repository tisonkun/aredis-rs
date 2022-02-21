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

mod del;
mod flushall;
mod get;
mod getrange;
mod mget;
mod mset;
mod msetnx;
mod ping;
mod set;
mod setrange;
mod strlen;

use bytes::{BufMut, BytesMut};
pub use del::Del;
pub use flushall::FlushAll;
pub use get::Get;
pub use getrange::GetRange;
pub use mget::MGet;
pub use mset::MSet;
pub use msetnx::MSetNx;
pub use ping::Ping;
pub use set::{Set, SetOption};
pub use setrange::SetRange;
pub use strlen::Strlen;

pub trait Command {
    fn as_bytes(&self) -> Vec<u8>;
}

fn args_to_bytes(args: Vec<&[u8]>) -> Vec<u8> {
    let mut result = BytesMut::new();
    result.put_slice(format!("*{}\r\n", args.len()).as_bytes());
    for arg in args {
        result.put_slice(format!("${}\r\n", arg.len()).as_bytes());
        result.put_slice(arg);
        result.put_slice("\r\n".as_bytes());
    }
    result.to_vec()
}
