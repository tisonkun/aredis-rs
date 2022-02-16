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

use bytes::Buf;
use num::BigInt;

use crate::{Error, Result};

#[derive(Debug)]
pub enum Model {
    Error(String),
    Status(String),
    Integer(i64),
    Double(f64),
    Nil,
    String(Vec<u8>),
    Array(Vec<Model>),
    Map(Vec<(Model, Model)>),
    Set(Vec<Model>),
    Bool(bool),
    Verb(Vec<u8>, Vec<u8>),
    Push(Vec<Model>),
    BigNum(num::BigInt),
}

pub fn check(cursor: &mut Cursor<&[u8]>) -> Result<()> {
    if !cursor.has_remaining() {
        return Err(Error::incomplete());
    }

    match cursor.get_u8() {
        b'-' | b'+' | b':' | b',' | b'(' | b'_' | b'#' => {
            readline(cursor)?;
        }
        b'$' | b'=' => {
            let len = readline(cursor)?;
            let len = atoi::atoi::<i64>(len).ok_or_else(|| Error::internal("malformed"))?;
            if len >= 0 {
                readline(cursor)?;
            }
        }
        b'~' | b'*' | b'>' => {
            let len = readline(cursor)?;
            let len = atoi::atoi::<i64>(len).ok_or_else(|| Error::internal("malformed"))?;
            for _ in 0..len {
                check(cursor)?;
            }
        }
        b'%' => {
            let len = readline(cursor)?;
            let len = atoi::atoi::<i64>(len).ok_or_else(|| Error::internal("malformed"))?;
            let len = len * 2;
            for _ in 0..len {
                check(cursor)?;
            }
        }
        b => return Err(Error::internal(format!("unknown type: {}", b))),
    }
    Ok(())
}

pub fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Model> {
    let model = match cursor.get_u8() {
        b'-' => {
            let error = readline(cursor)?;
            let error = String::from_utf8(error.to_vec())?;
            Ok(Model::Error(error))
        }
        b'+' => {
            let status = readline(cursor)?;
            let status = String::from_utf8(status.to_vec())?;
            Ok(Model::Status(status))
        }
        b':' => {
            let integer = readline(cursor)?;
            match atoi::atoi::<i64>(integer) {
                None => Err(Error::internal(format!("malformed integer: {:?}", integer))),
                Some(i) => Ok(Model::Integer(i)),
            }
        }
        b',' => {
            let double = readline(cursor)?;
            let double = String::from_utf8(double.to_vec())?;
            let double = double.parse::<f64>()?;
            Ok(Model::Double(double))
        }
        b'(' => {
            let bignum = readline(cursor)?;
            match BigInt::parse_bytes(bignum, 10) {
                None => Err(Error::internal(format!("malformed bignum: {:?}", bignum))),
                Some(i) => Ok(Model::BigNum(i)),
            }
        }
        b'_' => {
            let remaining = readline(cursor)?;
            if remaining.is_empty() {
                Ok(Model::Nil)
            } else {
                Err(Error::internal(format!("malformed nil: {:?}", remaining)))
            }
        }
        b'#' => {
            let remaining = readline(cursor)?;
            if remaining.len() != 1 {
                Err(Error::internal(format!("malformed bool: {:?}", remaining)))
            } else if remaining[0].to_ascii_lowercase() == b't' {
                Ok(Model::Bool(true))
            } else if remaining[0].to_ascii_lowercase() == b'f' {
                Ok(Model::Bool(false))
            } else {
                Err(Error::internal(format!("malformed bool: {:?}", remaining)))
            }
        }
        b'$' => {
            let len = readline(cursor)?;
            match atoi::atoi::<i64>(len) {
                None => Err(Error::internal(format!("malformed len: {:?}", len))),
                Some(len) if len < -1 => Err(Error::internal(format!("malformed len: {:?}", len))),
                Some(len) if len == -1 => Ok(Model::Nil),
                Some(_) => {
                    let next = readline(cursor)?;
                    Ok(Model::String(next.to_vec()))
                }
            }
        }
        b'=' => {
            let len = readline(cursor)?;
            match atoi::atoi::<i64>(len) {
                None => Err(Error::internal(format!("malformed len: {:?}", len))),
                Some(len) => {
                    if len == -1 {
                        Ok(Model::Nil)
                    } else if len < 4 {
                        Err(Error::internal(format!("malformed len: {:?}", len)))
                    } else {
                        let next = readline(cursor)?;
                        if next[3] != b':' {
                            let msg = format!("malformed verbatim string: {:?}", next);
                            Err(Error::internal(msg))
                        } else {
                            let (format, text) = next.split_at(3);
                            Ok(Model::Verb(format.to_vec(), text[1..].to_vec()))
                        }
                    }
                }
            }
        }
        t @ (b'~' | b'*' | b'>') => {
            let len = readline(cursor)?;
            match atoi::atoi::<i64>(len) {
                None => Err(Error::internal(format!("malformed len: {:?}", len))),
                Some(len) if len < -1 => Err(Error::internal(format!("malformed len: {:?}", len))),
                Some(len) if len == -1 => Ok(Model::Nil),
                Some(len) => {
                    let mut vec = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        vec.push(parse(cursor)?);
                    }
                    match t {
                        b'~' => Ok(Model::Set(vec)),
                        b'*' => Ok(Model::Array(vec)),
                        b'>' => Ok(Model::Push(vec)),
                        _ => unreachable!(),
                    }
                }
            }
        }
        b'%' => {
            let len = readline(cursor)?;
            match atoi::atoi::<i64>(len) {
                None => Err(Error::internal(format!("malformed len: {:?}", len))),
                Some(len) if len < -1 => Err(Error::internal(format!("malformed len: {:?}", len))),
                Some(len) if len == -1 => Ok(Model::Nil),
                Some(len) => {
                    let mut vec = Vec::with_capacity((2 * len) as usize);
                    for _ in 0..len {
                        let k = parse(cursor)?;
                        let v = parse(cursor)?;
                        vec.push((k, v));
                    }
                    Ok(Model::Map(vec))
                }
            }
        }
        b => Err(Error::internal(format!("unknown type: {}", b))),
    }?;

    Ok(model)
}

fn readline<'a>(cursor: &mut Cursor<&'a [u8]>) -> Result<&'a [u8]> {
    let start = cursor.position() as usize;
    let end = cursor.get_ref().len() - 1;

    for i in start..end {
        let inner = cursor.get_ref();
        if inner[i] == b'\r' && inner[i + 1] == b'\n' {
            cursor.set_position((i + 2) as u64);
            return Ok(&cursor.get_ref()[start..i]);
        }
    }

    Err(Error::incomplete())
}
