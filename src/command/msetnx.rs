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

use crate::command::{args_to_bytes, Command};

pub struct MSetNx {
    kvs: Vec<(Vec<u8>, Vec<u8>)>,
}

impl MSetNx {
    pub fn new(kvs: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        MSetNx { kvs }
    }
}

impl Command for MSetNx {
    fn as_bytes(&self) -> Vec<u8> {
        let mut args = vec!["MSETNX".as_bytes()];
        for kv in &self.kvs {
            args.push(kv.0.as_slice());
            args.push(kv.1.as_slice());
        }
        args_to_bytes(args)
    }
}
