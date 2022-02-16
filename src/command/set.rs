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

pub struct Set {
    key: Vec<u8>,
    value: Vec<u8>,
    expire: ExpireOption,
    exist: ExistOption,
    get: bool,
}

impl Command for Set {
    fn as_bytes(&self) -> Vec<u8> {
        let mut args = vec!["SET".as_bytes(), self.key.as_slice(), self.value.as_slice()];

        let expire; // lift expire duration/timestamp lifetime
        match self.expire {
            ExpireOption::None => {}
            ExpireOption::Ex(n) => {
                expire = n.to_string();
                args.push("EX".as_bytes());
                args.push(expire.as_bytes());
            }
            ExpireOption::Px(n) => {
                expire = n.to_string();
                args.push("PX".as_bytes());
                args.push(expire.as_bytes());
            }
            ExpireOption::Exat(n) => {
                expire = n.to_string();
                args.push("EXAT".as_bytes());
                args.push(expire.as_bytes());
            }
            ExpireOption::Pxat(n) => {
                expire = n.to_string();
                args.push("PXAT".as_bytes());
                args.push(expire.as_bytes());
            }
            ExpireOption::KeepTTL => args.push("KEEPTTL".as_bytes()),
        }

        match self.exist {
            ExistOption::None => {}
            ExistOption::Nx => args.push("NX".as_bytes()),
            ExistOption::Xx => args.push("XX".as_bytes()),
        }

        if self.get {
            args.push("GET".as_bytes())
        }

        args_to_bytes(args)
    }
}

pub struct SetOption {
    expire: ExpireOption,
    exist: ExistOption,
}

pub enum ExpireOption {
    None,
    Ex(u64),
    Px(u64),
    Exat(u64),
    Pxat(u64),
    KeepTTL,
}

pub enum ExistOption {
    None,
    Nx,
    Xx,
}

impl Default for SetOption {
    fn default() -> Self {
        SetOption {
            expire: ExpireOption::None,
            exist: ExistOption::None,
        }
    }
}

impl SetOption {
    pub fn ex(self, ex: u64) -> Self {
        SetOption {
            expire: ExpireOption::Ex(ex),
            ..self
        }
    }

    pub fn px(self, px: u64) -> Self {
        SetOption {
            expire: ExpireOption::Px(px),
            ..self
        }
    }

    pub fn exat(self, exat: u64) -> Self {
        SetOption {
            expire: ExpireOption::Exat(exat),
            ..self
        }
    }

    pub fn pxat(self, pxat: u64) -> Self {
        SetOption {
            expire: ExpireOption::Pxat(pxat),
            ..self
        }
    }

    pub fn keep_ttl(self) -> Self {
        SetOption {
            expire: ExpireOption::KeepTTL,
            ..self
        }
    }

    pub fn nx(self) -> Self {
        SetOption {
            exist: ExistOption::Nx,
            ..self
        }
    }

    pub fn xx(self) -> Self {
        SetOption {
            exist: ExistOption::Xx,
            ..self
        }
    }

    pub(crate) fn build(self, key: Vec<u8>, value: Vec<u8>, get: bool) -> Set {
        Set {
            key,
            value,
            expire: self.expire,
            exist: self.exist,
            get,
        }
    }
}
