use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use std::io::{Error, ErrorKind};
use std::str;

#[derive(Debug)]
struct Obj {
    offset: u32,
    genid: u32,
    sign: bool,
}
impl Obj {
    fn new(offset: u32, genid: u32, sign: bool) -> Obj {
        Obj {
            offset,
            genid,
            sign,
        }
    }
}
#[derive(Debug)]
pub struct Pdf {
    version: String,
    obj_list: Vec<Obj>,
}

impl Pdf {
    pub fn new() -> Pdf {
        Pdf {
            version: "".to_string(),
            obj_list: Vec::new(),
        }
    }
    pub fn open(path: &'static str) -> io::Result<Pdf> {
        let mut file = File::open(path).unwrap();
        println!("{:?}", file);
        let len: u64 = match file.seek(SeekFrom::End(0)) {
            Ok(len) => len,
            Err(_) => 0,
        };
        println!("file length {}", len);
        file.seek(SeekFrom::Start(0));
        let mut buf_reader = BufReader::new(file);
        let mut buf: Vec<u8> = Vec::with_capacity(10);
        buf_reader.read_until(b'\n', &mut buf);

        let len = buf.len() - 1;
        buf[len] = b'\0';
        let ver = String::from_utf8(buf).unwrap(); //str::from_utf8(&buf).unwrap();
        println!("version {}. {}", ver, ver.len());
        if &ver[0..5] == "%PDF-" {
            println!("match {}", &ver[5..]);
        }
        // startxref
        buf_reader.seek(SeekFrom::End(-32));

        // read tails
        let mut ref_offset: usize = 0;
        for i in 0..4 {
            let mut buffer = String::new();
            buf_reader.read_line(&mut buffer)?;
            if buffer == "startxref\n" {
                let mut buffer = String::new();
                buf_reader.read_line(&mut buffer)?;
                ref_offset = buffer.trim().parse().expect("??");
                break;
            }
            println!("{}", buffer);
        }
        println!("ref_start_oos {}", ref_offset);
        if ref_offset == 0 {
            return Err(Error::new(ErrorKind::Other, "ref_offset"));
        }
        // Err("?")
        buf_reader.seek(SeekFrom::Start(ref_offset as u64));
        let mut buffer = String::new();
        buf_reader.read_line(&mut buffer)?;
        if buffer != "xref\n" {
            return Err(Error::new(ErrorKind::Other, "ref sign"));
        }
        let mut pdf = Pdf::new();
        loop {
            let mut buffer = String::new();
            buf_reader.read_line(&mut buffer)?;
            if buffer == "trailer\n" {
                read_trailer(&mut pdf, &mut buf_reader);
                break;
            }
            let iter: Vec<&str> = buffer.split_whitespace().collect();
            let count: u32 = iter[1].parse().unwrap();
            for i in 0..count {
                buffer.clear();
                buf_reader.read_line(&mut buffer)?;
                let three: Vec<&str> = buffer.split_whitespace().collect();
                println!("{:?}", three);
                let offset: u32 = three[0].parse().expect("??");
                let genid: u32 = three[1].parse().expect("??");
                let sign: bool = match three[2] {
                    "n" => true,
                    _ => false,
                };
                let obj = Obj::new(offset, genid, sign);
                pdf.obj_list.push(obj);
            }
            // println!("{:?}", iter);
        }
        // here is trailer <</Size 20 /Root 3 0 R /Info 1 0 R>>

        Ok(pdf)
    }
}
fn read_trailer(pdf: &mut Pdf, buf_reader: &mut BufReader<File>) -> io::Result<usize> {
    read_dictonary(buf_reader);
    let mut buffer = String::new();
    buf_reader.read_line(&mut buffer)?;
    println!("{}", buffer);
    buf_reader.read_line(&mut buffer)?;
    println!("{}", buffer);

    Ok(0)
}

// << >>
fn read_dictonary(buf_reader: &mut BufReader<File>) -> io::Result<usize> {
    read_token(buf_reader);
    read_token(buf_reader);
    read_token(buf_reader);
    Ok(0)
}
// obj  endobj
fn read_object(buf_reader: &mut BufReader<File>) -> io::Result<usize> {
    let mut buffer = String::new();
    read_token(buf_reader);
    buf_reader.read_line(&mut buffer)?;
    println!("{}", buffer);
    Ok(0)
}
#[derive(Debug)]
enum Token {
    None,
    OBJ_BEGIN,
    OBJ_END,
    ARRAY_BEGIN,
    ARRAY_END,
    WORD,
    INTEGER(i32),
    DOUBLE(f64),
    NAME(String),
}

// lexer
fn read_token(buf_reader: &mut BufReader<File>) -> Result<Token, Error> {
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
            let t=  read_number(buf_reader);
            println!("{:?}",t);
        }
        _ => {
            buf_reader.seek(SeekFrom::Current(-1));
        }
    }
    loop {
        match buf_reader.read(&mut buf) {
            Ok(n) => ch = buf[0],
            _ => return Ok(Token::None), //TODO
        }
        match ch {
            b'%' => skip_comment(buf_reader),
            b'/' => {
                let t = read_name(buf_reader);
                println!("{:?}", t);
                return t;
            }
            _ => (),
        }
    }
    // println!("token:   {}",ver );

    Ok(Token::None)
}

fn read_number(buf_reader: &mut BufReader<File>) -> Result<Token, Error> {
    let mut num_buf: Vec<u8> = Vec::with_capacity(128);
    let mut is_real =false;
    let mut buf: [u8; 1] = [0];
    loop{
        buf_reader.read(&mut buf)?;
        let c = buf[0];
        if c== b'.'{
            is_real = true;
        }
        if is_number(c){
            num_buf.push(c);
        }else{
            break;
        }
    }
    // parse number
    match String::from_utf8(num_buf){
        Ok(s) =>{
            if is_real{
                let dr = s.parse::<f64>();
                if let Ok(n) = dr{
                    return Ok(Token::DOUBLE(n));
                }
            }else{
                 let dr = s.parse::<i32>();
                if let Ok(n) = dr{
                    return Ok(Token::INTEGER(n));
                }
            }
        }
        Err(_) => return Err(Error::new(ErrorKind::Other,"from_utf8")),
    }
   
    Ok(Token::INTEGER(0))
}
fn read_name(buf_reader: &mut BufReader<File>) -> Result<Token, Error> {
    let mut name_buf: Vec<u8> = Vec::with_capacity(128);

    let mut buf: [u8; 1] = [0];
    let mut hex = false;
    loop {
        buf_reader.read(&mut buf)?;
        let c = buf[0];
        match c {
            c if is_white(c) => break,
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
        Ok(s) => Ok(Token::NAME(s)),
        Err(_) => Err(Error::new(ErrorKind::Other, "utf8_to_str")),
    }

    // Ok(Token::NAME(s))
}
fn skip_comment(buf_reader: &mut BufReader<File>) {}
fn skip_white(buf_reader: &mut BufReader<File>) {
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
fn is_white(ch: u8) -> bool {
    let ret = match ch {
        b' ' => true,
        b'\n' => true,
        b'\r' => true,
        b'\t' => true,
        _ => false,
    };
    ret
}

fn is_number(ch: u8) -> bool {
    // println!("is_number({})",ch);
    let ret = match ch {
        b'+' => true,
        b'-' => true,
        b'.' => true,
        b'0'...b'9' =>true,
        _ => false,
    };
    ret
}

/*
bool
 true false
number
[+-]nnn.nnn
name  max127
/xxx
xxx:  0x21-0x7E
xxx:  #hex 十六进制

string
xxxx ：
xxx： \ddd  八进制
xxx: <dd> 十六进制
\n \r \t \b（退格） \f(换页) \（  \), \\

dict
<< >>
array
[]


*/
