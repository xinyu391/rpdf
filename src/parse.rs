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
    ERROR(ParseError),
}
#[derive(Debug)]
pub struct ParseError{
    msg :String
}
impl ParseError{
    pub fn new(msg:&str)->Self{
        ParseError{msg:String::from(msg)}
    }
}
impl From<io::Error> for ParseError{
    fn from(err:io::Error)-> ParseError{
        ParseError{msg:String::from("")}
    }
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
pub fn peek_token(buf_reader: &mut BufReader<File>) -> Token{
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

    loop{
        match read_byte(buf_reader){
            Ok(c)=>{
                match c{
                    c if is_white(c) => {skip_white(buf_reader);},
                    b'%' =>skip_comment(buf_reader),
                    b'/' => {
                        return read_name(buf_reader);
                    }
                    b'<' =>{
                        if let Ok(b'<') = read_byte(buf_reader){
                            return Token::DICT_BEGIN;
                        }else{
                            buf_reader.seek(SeekFrom::Current(-1));
                            return read_hex_string(buf_reader);
                        }
                    }
                    b'>' =>{
                        match  read_byte(buf_reader){
                            Ok(b'>') =>{
                                return Token::DICT_END;
                            }
                            Err(e) =>{
                                return Token::ERROR(e);
                            }
                            _ =>{
                                buf_reader.seek(SeekFrom::Current(-1));
                            }
                        }
                    }
                    b'(' => return read_string(buf_reader),
                    b')' => {
                        // should no be here
                        panic!("should not be herer");
                    }
                    b'[' =>return Token::ARRAY_BEGIN,
                    b']' =>return Token::ARRAY_BEGIN,
                    c if is_number(c) =>return read_number(buf_reader, c),
                    _=>{
                        //what ?
                        buf_reader.seek(SeekFrom::Current(-1));
                        println!(" helll no ");
                        if let Token::NAME(s)= read_name(buf_reader){

                            println!(" helll no {}",s);
                            return to_token(s);
                        }else{
                            return Token::ERROR(ParseError::new("whate hell? "));
                        }
                    }

                }
            }
            Err(e)=> return Token::ERROR(e),
        }
    }

}
fn to_token(s: String) -> Token{
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
pub fn read_number(buf_reader: &mut BufReader<File>, c :u8) -> Token {
    let mut num_buf: Vec<u8> = Vec::with_capacity(128);
    let mut is_real = false;
    num_buf.push(c);
    if c == b'.'{
        is_real  = true;
    }
    loop {

        match read_byte(buf_reader){
            Ok(c)=>{
                match c{
                    c if is_white(c) =>break,
                    c if is_delimiter(c) =>{
                         buf_reader.seek(SeekFrom::Current(-1));
                        break;
                    }
                    b'.' =>{
                        is_real = true;
                        num_buf.push(c);
                    }
                    c if is_number(c) =>{
                        num_buf.push(c);
                    }
                    _ =>{
                        return Token::ERROR(ParseError::new("wrong number")); 
                    }
                }
            }   
            Err(e)=>{
                return Token::ERROR(ParseError::new("wrong number"));
            } 
        }
    }
    // parse number
    match String::from_utf8(num_buf) {
        Ok(s) => {
            println!("xxxxxx read number -> {}",s);
            if is_real {
                let dr = s.parse::<f64>();
                if let Ok(n) = dr {
                    return   Token::FLOAT(n);
                }
            } else {
                let dr = s.parse::<i32>();
                if let Ok(n) = dr {
                    return  Token::INTEGER(n);
                }
            }
            return Token::ERROR(ParseError::new("from_utf8 read_number"));
        }
        Err(_) =>   Token::ERROR(ParseError::new("from_utf8")),
    }
}


pub fn read_hex_string(buf_reader: &mut BufReader<File>) -> Token {
    let s = String::new();
    Token::STRING(s)
}
pub fn read_string(buf_reader: &mut BufReader<File>) -> Token {
    let mut name_buf: Vec<u8> = Vec::new();

    let mut buf: [u8; 1] = [0];
    let mut hex = false;
    let mut backslash = false;

    skip_white(buf_reader);
    if let Ok(1) = buf_reader.read(&mut buf) {
        let c = buf[0];
        match c {
            b'(' => {
                name_buf.push(c);
            }
            b'<' => {
                name_buf.push(c);
                hex = true;
            }
            b'R' => {
                return Token::STRING(String::from("R"));
            }
            _ => {
                panic!("wrong char read_string {}", c);
            }
        }
    } else {
        return Token::ERROR(ParseError::new("read error in read_string"));
    }
    loop {
        // read until to ')'
        if let Ok(1) = buf_reader.read(&mut buf) {
            let c = buf[0];
            // println!("{}",c);
            match c {
                b'\\' => {
                    // 转义
                    backslash = true;
                }
                c if backslash => {
                    name_buf.push(c);
                    backslash = false;
                }
                b')' => {
                    name_buf.push(c);
                    break;
                }
                _ => {
                    name_buf.push(c);
                }
            }
        }
    }
    match String::from_utf8(name_buf) {
        Ok(s) => Token::STRING(s),
        Err(_) => Token::ERROR(ParseError::new("read error in read_string")),
    }
}
pub fn read_name(buf_reader: &mut BufReader<File>) -> Token {
    let mut name_buf: Vec<u8> = Vec::with_capacity(128);

    loop {
        match read_byte(buf_reader){
            Ok(c)=> {
                match c{
                    c if is_white(c) =>break,
                    c if is_delimiter(c) =>{
                         buf_reader.seek(SeekFrom::Current(-1));
                        break;
                    }

                    b'#'=>{
                            //read two byte
                            if let Ok(c0) = read_byte(buf_reader){
                                 if let Ok(c1) = read_byte(buf_reader){
                                     // c0,c1 -> c
                                      if let Ok(c0) = hex_to_char(c0){
                                          if let Ok(c1)= hex_to_char(c1){
                                            name_buf.push(c0+c1);
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
            Err(e)=>{
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
fn  hex_to_char(c0:u8)->Result<u8,ParseError>{

    match c0{
        b'0'...b'9' => Ok(c0- b'0'), 
        b'a'...b'z' =>  Ok(c0- b'a'+ 10),
        b'A'...b'Z' =>  Ok(c0- b'A'+ 10),
        _ => Err(ParseError::new("not hex ")),
    }
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
//'\x00':case'\x09':case'\x0a':case'\x0c':case'\x0d':case'\x20'
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
