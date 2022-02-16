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

pub struct Del {
    keys: Vec<Vec<u8>>,
}

impl Del {
    pub fn new(keys: Vec<Vec<u8>>) -> Self {
        Del { keys }
    }
}

impl Command for Del {
    fn as_bytes(&self) -> Vec<u8> {
        let mut args = vec!["DEL".as_bytes()];
        for key in &self.keys {
            args.push(key.as_slice());
        }
        args_to_bytes(args)
    }
}
