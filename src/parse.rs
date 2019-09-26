use std::collections::HashMap;
use std::fmt;
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
    ERROR(ParseError),
}
#[derive(Debug)]
pub struct ParseError {
    msg: String,
}
impl ParseError {
    pub fn new(msg: &str) -> Self {
        ParseError {
            msg: String::from(msg),
        }
    }
}
impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError {
            msg: String::from(""),
        }
    }
}

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

impl fmt::Debug for Value{
    fn fmt(&self, f:&mut fmt::Formatter<'_>)->fmt::Result{
        // write!(f,"")
        match self{
            Value::INTEGER(v) => write!(f,"{}", v),
            Value::BOOL(v) => write!(f,"{}", v),
            Value::REF(v0, v1) =>write!(f,"{}_{}_R", v0,v1),
            Value::NAME(v)=>write!(f,"/{}",v),
            Value::STRING(v)=>write!(f,"\"{}\"",v),
            Value::NULL =>write!(f,"null"),
            Value::FLOAT(v)=>write!(f,"{}",v),
            Value::ARRAY(v) =>write!(f,"{:?}",v),
            Value::DICT(v)=>write!(f,"<<{:?}>>",v),
        }
    }
}
#[derive(Debug)]
pub struct Dict {
    map: HashMap<String, Value>,
}
// impl fmt::Debug for Dict{
//     fn fmt(&self, f:&mut fmt::Formatter<'_>)->fmt::Result{
//         write!(f,"<<");
//         for (key, value) in self.map{
//             write!(f,"{}:{}",key, value);
//         }
//         write!(f,">>");
//     }
// }
impl Dict {
    pub fn new() -> Self {
        Dict {
            map: HashMap::new(),
        }
    }
    pub fn push(&mut self, key: String, val: Value) {
        self.map.insert(key, val);
    }
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(key)
    }
}
pub struct Stream {
    pub data: Vec<u8>,
}
impl Stream {
    pub fn new(data: Vec<u8>) -> Self {
        Stream { data }
    }
    pub fn decode(&mut self){

    }
}
impl fmt::Debug for Stream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stream{{{} bytes}}", self.data.len())
    }
}

pub fn peek_token(buf_reader: &mut BufReader<File>) -> Token {
    if let Ok(pos) = buf_reader.seek(SeekFrom::Current(0)) {
        let tk = read_token(buf_reader);
        buf_reader.seek(SeekFrom::Start(pos));
        return tk;
    }
    return Token::ERROR(ParseError::new("seek error"));
}

fn read_byte(buf_reader: &mut BufReader<File>) -> Result<u8, ParseError> {
    let mut buf: [u8; 1] = [0];
    if let Ok(1) = buf_reader.read(&mut buf) {
        return Ok(buf[0]);
    }
    Err(ParseError::new("read_byte error "))
}
// lexer
pub fn read_token(buf_reader: &mut BufReader<File>) -> Token {
    loop {
        match read_byte(buf_reader) {
            Ok(c) => {
                match c {
                    c if is_white(c) => {
                        skip_white(buf_reader);
                    }
                    b'%' => skip_comment(buf_reader),
                    b'/' => {
                        return read_name(buf_reader);
                    }
                    b'<' => {
                        if let Ok(b'<') = read_byte(buf_reader) {
                            return Token::DICT_BEGIN;
                        } else {
                            unread_bytes(buf_reader, 1);
                            return read_hex_string(buf_reader);
                        }
                    }
                    b'>' => match read_byte(buf_reader) {
                        Ok(b'>') => {
                            return Token::DICT_END;
                        }
                        Err(e) => {
                            return Token::ERROR(e);
                        }
                        _ => {
                            unread_bytes(buf_reader, 1);
                        }
                    },
                    b'(' => return read_string(buf_reader),
                    b')' => {
                        // should no be here
                        panic!("should not be herer");
                    }
                    b'[' => return Token::ARRAY_BEGIN,
                    b']' => return Token::ARRAY_END,
                    c if is_number(c) => return read_number(buf_reader, c),
                    _ => {
                        //what ?
                        unread_bytes(buf_reader, 1);
                        if let Token::NAME(s) = read_name(buf_reader) {
                            return to_token(s);
                        } else {
                            return Token::ERROR(ParseError::new("whate hell? "));
                        }
                    }
                }
            }
            Err(e) => return Token::ERROR(e),
        }
    }
}
fn to_token(s: String) -> Token {
    if s == "true" {
        return Token::BOOL(true);
    } else if s == "false" {
        return Token::BOOL(false);
    } else if s == "null" {
        return Token::NULL;
    } else if s == "obj" {
        return Token::OBJ_BEGIN;
    } else if s == "endobj" {
        return Token::OBJ_END;
    } else if s == "stream" {
        return Token::STREAM_BEGIN;
    } else if s == "endstream" {
        return Token::STREAM_END;
    } else if s == "R" {
        return Token::R;
    }
    return Token::ERROR(ParseError::new("to_token"));
}
pub fn read_stream(buf_reader: &mut BufReader<File>, size: usize) -> Result<Stream, ParseError> {
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    buf.resize(size, 0);
    match buf_reader.read_exact(buf.as_mut_slice()) {
        Ok(()) => Ok(Stream::new(buf)),
        Err(e) => Err(ParseError::new("read error")),
    }
}
pub fn read_number(buf_reader: &mut BufReader<File>, c: u8) -> Token {
    let mut num_buf: Vec<u8> = Vec::with_capacity(128);
    let mut is_real = false;
    num_buf.push(c);
    if c == b'.' {
        is_real = true;
    }
    loop {
        match read_byte(buf_reader) {
            Ok(c) => match c {
                c if is_white(c) => break,
                c if is_delimiter(c) => {
                    unread_bytes(buf_reader, 1);
                    break;
                }
                b'.' => {
                    is_real = true;
                    num_buf.push(c);
                }
                c if is_number(c) => {
                    num_buf.push(c);
                }
                _ => {
                    return Token::ERROR(ParseError::new("wrong number"));
                }
            },
            Err(e) => {
                return Token::ERROR(ParseError::new("wrong number"));
            }
        }
    }
    // parse number
    match String::from_utf8(num_buf) {
        Ok(s) => {
            // println!("xxxxxx read number -> {}", s);
            if is_real {
                let dr = s.parse::<f64>();
                if let Ok(n) = dr {
                    return Token::FLOAT(n);
                }
            } else {
                let dr = s.parse::<i32>();
                if let Ok(n) = dr {
                    return Token::INTEGER(n);
                }
            }
            return Token::ERROR(ParseError::new("from_utf8 read_number"));
        }
        Err(_) => Token::ERROR(ParseError::new("from_utf8")),
    }
}

pub fn read_hex_string(buf_reader: &mut BufReader<File>) -> Token {
    // read until >
    let mut buf: Vec<u8> = Vec::new();
    if let Ok(n) = buf_reader.read_until(b'>', &mut buf) {
        if let Ok(s) = String::from_utf8(buf) {
            return Token::STRING(s);
        }
    }
    Token::ERROR(ParseError::new("read_hex_string from_utf8"))
}
pub fn read_string(buf_reader: &mut BufReader<File>) -> Token {
    let mut name_buf: Vec<u8> = Vec::new();
    // name_buf.push(b'(');
    let mut count: u32 = 1;
    loop {
        // read until to ')'
        match read_byte(buf_reader) {
            Ok(c) => {
                match c {
                    b'\\' => {
                        //blackslash
                        if let Ok(c1) = read_byte(buf_reader) {
                            match c1 {
                                b'n' => name_buf.push(b'\n'),
                                b'r' => name_buf.push(b'\r'),
                                b't' => name_buf.push(b'\t'),
                                b'b' => {
                                    name_buf.pop();
                                }
                                b'\n' => (),
                                b'\r' => (),
                                b'f' => {
                                    // TODO ??? ignore.
                                    //    name_buf.push(b'\f'),
                                }
                                b'(' => name_buf.push(b'('),
                                b')' => name_buf.push(b')'),
                                b'\\' => name_buf.push(c1),
                                b'0'...b'7' => {
                                    // todo at most three number 0-7
                                    if let Ok(c2) = read_byte(buf_reader) {
                                        match c2 {
                                            b'0'...b'7' => {
                                                if let Ok(c3) = read_byte(buf_reader) {
                                                    match c3 {
                                                        b'0'...b'7' => {
                                                            // combine c1 c2 c3
                                                        }
                                                        _ => {
                                                            unread_bytes(buf_reader, 1);
                                                            // c1 c2 TODO
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                               unread_bytes(buf_reader, 1);
                                                // c1
                                            }
                                        }
                                    }
                                }
                                _ => (), // should not be here ignore
                            }
                        }
                    }
                    b'(' => count += 1,
                    b')' => {
                        count -= 1;
                        if count == 0 {
                            break;
                        }
                        name_buf.push(c);
                    }
                    _ => {
                        name_buf.push(c);
                    }
                }
            }
            Err(e) => {}
        }
    }
    match String::from_utf8(name_buf) {
        Ok(s) => Token::STRING(s),
        Err(_) => Token::ERROR(ParseError::new("read error in read_string")),
    }
}
fn unread_bytes(buf_reader: &mut BufReader<File>, n :i64){
    buf_reader.seek(SeekFrom::Current(-n));
}
pub fn read_name(buf_reader: &mut BufReader<File>) -> Token {
    let mut name_buf: Vec<u8> = Vec::with_capacity(128);

    loop {
        match read_byte(buf_reader) {
            Ok(c) => {
                match c {
                    c if is_white(c) => break,
                    c if is_delimiter(c) => {
                        unread_bytes(buf_reader, 1);
                        break;
                    }

                    b'#' => {
                        //read two byte
                        if let Ok(c0) = read_byte(buf_reader) {
                            if let Ok(c1) = read_byte(buf_reader) {
                                // c0,c1 -> c
                                if let Ok(c0) = hex_to_char(c0) {
                                    if let Ok(c1) = hex_to_char(c1) {
                                        name_buf.push(c0 + c1);
                                        continue;
                                    }
                                }
                            }
                        }
                        break;
                    }
                    _ => name_buf.push(c),
                }
            }
            Err(e) => {
                break;
            }
        }
    }
    match String::from_utf8(name_buf) {
        Ok(s) => Token::NAME(s),
        Err(e) => Token::ERROR(ParseError::new("from_utf8 read name")),
    }

    // Ok(Token::NAME(s))
}
fn hex_to_char(c0: u8) -> Result<u8, ParseError> {
    match c0 {
        b'0'...b'9' => Ok(c0 - b'0'),
        b'a'...b'z' => Ok(c0 - b'a' + 10),
        b'A'...b'Z' => Ok(c0 - b'A' + 10),
        _ => Err(ParseError::new("not hex ")),
    }
}
pub fn skip_comment(buf_reader: &mut BufReader<File>) {
    // read until end of line
    let mut buf: [u8; 1] = [0];
    loop {
        match buf_reader.read(&mut buf) {
            Ok(1) => match buf[0] {
                b'\n' => break,
                b'\r' => break,
                _ => (),
            },
            _ => break,
        }
    }
}
pub fn skip_white(buf_reader: &mut BufReader<File>) {
    let mut buf: [u8; 1] = [0];
    loop {
        match buf_reader.read(&mut buf) {
            Ok(c) => {
                if !is_white(buf[0]) {
                    unread_bytes(buf_reader, 1);
                    break;
                }
            }
            _ => break,
        }
    }
}
//'\x00':case'\x09':case'\x0a':case'\x0c':case'\x0d':case'\x20'
pub fn is_white(ch: u8) -> bool {
    match ch {
        b'\0' => true,
        b'\t' => true,
        b'\r' => true,
        0x0c => true, // form feed 换页符号
        b'\n' => true,
        b' ' => true,
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
