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

use std::{cmp::Ordering, io::Cursor};

use bytes::Buf;
use num::BigInt;

use crate::error::{ParseError, ParseResult};

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

pub fn check(cursor: &mut Cursor<&[u8]>) -> ParseResult<()> {
    if !cursor.has_remaining() {
        return Err(ParseError::EndOfStream);
    }

    match cursor.get_u8() {
        b'-' | b'+' | b':' | b',' | b'(' | b'_' | b'#' => {
            readline(cursor)?;
        }
        b'$' | b'=' => {
            let len = readline(cursor)?;
            let len = parse_int::<i64>(len)?;
            if len >= 0 {
                readline(cursor)?;
            }
        }
        b'~' | b'*' | b'>' => {
            let len = readline(cursor)?;
            let len = parse_int::<i64>(len)?;
            for _ in 0..len {
                check(cursor)?;
            }
        }
        b'%' => {
            let len = readline(cursor)?;
            let len = parse_int::<i64>(len)?;
            let len = len * 2;
            for _ in 0..len {
                check(cursor)?;
            }
        }
        b => return Err(ParseError::Other(format!("unknown type: {}", b))),
    }
    Ok(())
}

pub fn parse(cursor: &mut Cursor<&[u8]>) -> ParseResult<Model> {
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
            let integer = parse_int::<i64>(integer)?;
            Ok(Model::Integer(integer))
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
                None => Err(ParseError::Other(format!("malformed bignum: {:?}", bignum))),
                Some(i) => Ok(Model::BigNum(i)),
            }
        }
        b'_' => {
            let remaining = readline(cursor)?;
            if remaining.is_empty() {
                Ok(Model::Nil)
            } else {
                Err(ParseError::Other(format!("malformed nil: {:?}", remaining)))
            }
        }
        b'#' => {
            let b = readline(cursor)?;
            if b.len() != 1 {
                Err(ParseError::Other(format!("malformed bool: {:?}", b)))
            } else if b[0].to_ascii_lowercase() == b't' {
                Ok(Model::Bool(true))
            } else if b[0].to_ascii_lowercase() == b'f' {
                Ok(Model::Bool(false))
            } else {
                Err(ParseError::Other(format!("malformed bool: {:?}", b)))
            }
        }
        b'$' => {
            let len = readline(cursor)?;
            let len = parse_int::<i64>(len)?;
            match len.cmp(&-1) {
                Ordering::Less => Err(ParseError::Other(format!("malformed len: {:?}", len))),
                Ordering::Equal => Ok(Model::Nil),
                Ordering::Greater => Ok(Model::String(readline(cursor)?.to_vec())),
            }
        }
        b'=' => {
            let len = readline(cursor)?;
            let len = parse_int::<i64>(len)?;
            if len == -1 {
                Ok(Model::Nil)
            } else if len < 4 {
                Err(ParseError::Other(format!("malformed len: {:?}", len)))
            } else {
                let next = readline(cursor)?;
                if next[3] != b':' {
                    Err(ParseError::Other(format!(
                        "malformed verbatim string: {:?}",
                        next
                    )))
                } else {
                    let (format, text) = next.split_at(3);
                    Ok(Model::Verb(format.to_vec(), text[1..].to_vec()))
                }
            }
        }
        t @ (b'~' | b'*' | b'>') => {
            let len = readline(cursor)?;
            let len = parse_int::<i64>(len)?;
            match len.cmp(&-1) {
                Ordering::Less => Err(ParseError::Other(format!("malformed len: {:?}", len))),
                Ordering::Equal => Ok(Model::Nil),
                Ordering::Greater => {
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
            let len = parse_int::<i64>(len)?;
            match len.cmp(&-1) {
                Ordering::Less => Err(ParseError::Other(format!("malformed len: {:?}", len))),
                Ordering::Equal => Ok(Model::Nil),
                Ordering::Greater => {
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
        b => Err(ParseError::Other(format!("unknown type: {}", b))),
    }?;

    Ok(model)
}

fn readline<'a>(cursor: &mut Cursor<&'a [u8]>) -> ParseResult<&'a [u8]> {
    let start = cursor.position() as usize;
    let end = cursor.get_ref().len() - 1;

    for i in start..end {
        let inner = cursor.get_ref();
        if inner[i] == b'\r' && inner[i + 1] == b'\n' {
            cursor.set_position((i + 2) as u64);
            return Ok(&cursor.get_ref()[start..i]);
        }
    }

    Err(ParseError::EndOfStream)
}

fn parse_int<T: atoi::FromRadix10SignedChecked>(i: &[u8]) -> ParseResult<T> {
    match atoi::atoi::<T>(i) {
        Some(i) => Ok(i),
        None => Err(ParseError::Other(format!("malformed integer: {:?}", i))),
    }
}
