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

pub struct Get {
    key: Vec<u8>,
}

impl Get {
    pub fn new(key: Vec<u8>) -> Self {
        Get { key }
    }
}

impl Command for Get {
    fn as_bytes(&self) -> Vec<u8> {
        args_to_bytes(vec!["GET".as_bytes(), self.key.as_slice()])
    }
}
