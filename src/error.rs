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

use std::fmt::Formatter;

#[derive(Debug, Copy, Clone)]
pub enum ErrorCode {
    Incomplete,
    Internal,
    Io,
}

impl ErrorCode {
    pub fn is_incomplete(&self) -> bool {
        matches!(self, ErrorCode::Incomplete)
    }
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
    message: String,
}

impl Error {
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn new(code: ErrorCode, message: String) -> Self {
        Self { code, message }
    }

    pub fn incomplete() -> Self {
        Self {
            code: ErrorCode::Incomplete,
            message: "".to_string(),
        }
    }

    pub fn internal<T: ToString>(message: T) -> Self {
        Self {
            code: ErrorCode::Internal,
            message: message.to_string(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self {
            code: ErrorCode::Io,
            message: e.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::internal(e)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(e: std::num::ParseFloatError) -> Self {
        Self::internal(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("code", &self.code)
            .field("message", &self.message)
            .finish()
    }
}

impl std::error::Error for Error {}
