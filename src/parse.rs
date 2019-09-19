use std::collections::HashMap;
use std::fs::File;

use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub enum Token {
    None,
    OBJ_BEGIN,
    OBJ_END,
    ARRAY_BEGIN,
    ARRAY_END,
    STREAM_BEGIN,
    STREAM_END,
    R,
    WORD,
    DICT_END,
    DICT_BEGIN,
    BOOL(bool),
    NULL,
    INTEGER(i32),
    FLOAT(f64),
    NAME(String),
    STRING(String),
}

#[derive(Debug)]
pub enum Value {
    INTEGER(i32),
    BOOL(bool),
    REF(i32, i32),
    NAME(String),
    STRING(String),
    NULL,
    FLOAT(f64),
    ARRAY(Vec<Value>),
    DICT(Dict),
}
#[derive(Debug)]
pub struct Dict {
    map: HashMap<String, Value>,
}
impl Dict {
    pub fn new() -> Self {
        Dict {
            map: HashMap::new(),
        }
    }
    pub fn push(&mut self, key: String, val: Value) {
        self.map.insert(key, val);
    }
}
pub fn peek_token(buf_reader: &mut BufReader<File>) -> Result<Token, Error> {
    if let Ok(pos) = buf_reader.seek(SeekFrom::Current(0)) {
        let tk = read_token(buf_reader);
        buf_reader.seek(SeekFrom::Start(pos));
        return tk;
    }
    return Err(Error::new(ErrorKind::Other, "seek error"));
}
// lexer
pub fn read_token(buf_reader: &mut BufReader<File>) -> Result<Token, Error> {
    let mut buf: [u8; 1] = [0];
    let mut ch: u8;
    match buf_reader.read(&mut buf) {
        Ok(1) => ch = buf[0],
        _ => return Err(Error::new(ErrorKind::Other, "read error")),
    }
    // 过滤空白符
    match ch {
        ch if is_white(ch) => skip_white(buf_reader),
        ch if is_number(ch) => {
            buf_reader.seek(SeekFrom::Current(-1));
            let t = read_number(buf_reader);
            // println!("read number {:?}",t);
            return t;
        }
        _ => {
            buf_reader.seek(SeekFrom::Current(-1));
        }
    }
    let mut last_is_bth = false;
    let mut last_is_lth = false;
    loop {
        match buf_reader.read(&mut buf) {
            Ok(n) => ch = buf[0],
            _ => return Ok(Token::NULL), //TODO
        }
        match ch {
            b'%' => skip_comment(buf_reader),
            b'/' => {
                let t = read_name(buf_reader);
                // println!("{:?}", t);
                match t {
                    Ok(s) => return Ok(Token::NAME(s)),
                    Err(e) => return Err(e),
                };
            }
            b'<' => {
                if last_is_lth {
                    return Ok(Token::DICT_BEGIN);
                } else {
                    last_is_lth = true;
                }
            }
            b'>' => {
                if last_is_bth {
                    return Ok(Token::DICT_END);
                } else {
                    last_is_bth = true;
                }
            }
            _ => {
                buf_reader.seek(SeekFrom::Current(-1));
                let t = read_name(buf_reader);
                //println!("_{:?}", t);
                match t {
                    Ok(s) => return to_token(s),
                    Err(e) => return Err(e),
                }
            }
        }
    }
    // println!("token:   {}",ver );

    Ok(Token::None)
}
fn to_token(s: String) -> Result<Token, Error> {
    if s == "true" {
        return Ok(Token::BOOL(true));
    } else if s == "false" {
        return Ok(Token::BOOL(false));
    } else if s == "null" {
        return Ok(Token::NULL);
    } else if s == "obj" {
        return Ok(Token::OBJ_BEGIN);
    } else if s == "endobj" {
        return Ok(Token::OBJ_END);
    } else if s == "stream" {
        return Ok(Token::STREAM_BEGIN);
    } else if s == "endstream" {
        return Ok(Token::STREAM_END);
    } else if s == "R" {
        return Ok(Token::R);
    }
    return Err(Error::new(ErrorKind::Other, s));
}
pub fn read_number(buf_reader: &mut BufReader<File>) -> Result<Token, Error> {
    let mut num_buf: Vec<u8> = Vec::with_capacity(128);
    let mut is_real = false;
    let mut buf: [u8; 1] = [0];
    loop {
        buf_reader.read(&mut buf)?;
        let c = buf[0];
        if c == b'.' {
            is_real = true;
        }
        if is_number(c) {
            num_buf.push(c);
        } else {
            break;
        }
    }
    // parse number
    match String::from_utf8(num_buf) {
        Ok(s) => {
            if is_real {
                let dr = s.parse::<f64>();
                if let Ok(n) = dr {
                    return Ok(Token::FLOAT(n));
                }
            } else {
                let dr = s.parse::<i32>();
                if let Ok(n) = dr {
                    return Ok(Token::INTEGER(n));
                }
            }
        }
        Err(_) => return Err(Error::new(ErrorKind::Other, "from_utf8")),
    }
    Ok(Token::INTEGER(0))
}

pub fn read_name(buf_reader: &mut BufReader<File>) -> Result<String, Error> {
    let mut name_buf: Vec<u8> = Vec::with_capacity(128);

    let mut buf: [u8; 1] = [0];
    let mut hex = false;
    loop {
        buf_reader.read(&mut buf)?;
        let c = buf[0];
        match c {
            c if is_white(c) => break,
            c if is_delimiter(c) => {
                buf_reader.seek(SeekFrom::Current(-1));
                break;
            }
            b'#' => hex = true,
            _ => {
                if hex {
                } else {
                    name_buf.push(c);
                }
            }
        }
        //  match buf_reader.read(&mut buf){
        //     Ok(c) => match buf[0]{
        //         b'#' => (),
        //         c=> name_buf.push(c),
        //     }
        //     _ => break,
        // }
    }
    match String::from_utf8(name_buf) {
        Ok(s) => Ok(s),
        Err(_) => Err(Error::new(ErrorKind::Other, "utf8_to_str")),
    }

    // Ok(Token::NAME(s))
}
pub fn skip_comment(buf_reader: &mut BufReader<File>) {
    //TODO
}
pub fn skip_white(buf_reader: &mut BufReader<File>) {
    let mut buf: [u8; 1] = [0];
    loop {
        match buf_reader.read(&mut buf) {
            Ok(c) => {
                if !is_white(buf[0]) {
                    buf_reader.seek(SeekFrom::Current(-1));
                    break;
                }
            }
            _ => break,
        }
    }
}
pub fn is_white(ch: u8) -> bool {
    match ch {
        b' ' => true,
        b'\n' => true,
        b'\r' => true,
        b'\t' => true,
        _ => false,
    }
}

fn is_delimiter(c: u8) -> bool {
    // 空白，
    match c {
        b'(' => true,
        b')' => true,
        b'<' => true,
        b'>' => true,
        b'[' => true,
        b']' => true,
        b'{' => true,
        b'}' => true,
        b'/' => true,
        b'%' => true,
        _ => false,
    }
}

pub fn is_number(ch: u8) -> bool {
    // println!("is_number({})",ch);
    let ret = match ch {
        b'+' => true,
        b'-' => true,
        b'.' => true,
        b'0'...b'9' => true,
        _ => false,
    };
    ret
}
