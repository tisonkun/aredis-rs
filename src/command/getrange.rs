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

pub struct GetRange {
    key: Vec<u8>,
    start: i64,
    end: i64,
}

impl GetRange {
    pub fn new(key: Vec<u8>, start: i64, end: i64) -> Self {
        GetRange { key, start, end }
    }
}

impl Command for GetRange {
    fn as_bytes(&self) -> Vec<u8> {
        args_to_bytes(vec![
            "GETRANGE".as_bytes(),
            self.key.as_slice(),
            self.start.to_string().as_bytes(),
            self.end.to_string().as_bytes(),
        ])
    }
}
