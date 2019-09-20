use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use std::io::{Error, ErrorKind};
use std::str;
use std::vec::*;

#[path = "parse.rs"]
mod parse;
use parse::*;

#[derive(Debug)]
struct Obj {
    id: u32,
    offset: u32,
    genid: u32,
    used: bool,
    // box real data
}
impl Obj {
    fn new(id: u32, offset: u32, genid: u32, used: bool) -> Obj {
        Obj {
            id,
            offset,
            genid,
            used,
        }
    }
    fn fill() -> bool {
        false
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
            let iter: Vec<u32> = buffer
                .split_whitespace()
                .map(|x| x.parse::<u32>().unwrap())
                .collect();
            let mut oid = iter[0]; //.parse().unwrap();
            let count = iter[1]; //.parse().unwrap();
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
                let obj = Obj::new(oid, offset, genid, sign);
                pdf.obj_list.push(obj);
                oid += 1;
            }
            // println!("{:?}", iter);
        }
        // read all obj
        read_objects(&mut pdf, &mut buf_reader);

        Ok(pdf)
    }
}
fn read_objects(pdf: &mut Pdf, buf_reader: &mut BufReader<File>) {
    for obj in &pdf.obj_list {
        println!("{:?}", obj);
        if obj.used {
            buf_reader.seek(SeekFrom::Start(obj.offset as u64));
            read_object(buf_reader);
        }
    }
}
fn read_trailer(pdf: &mut Pdf, buf_reader: &mut BufReader<File>) -> io::Result<usize> {
    let dict = read_dictonary(buf_reader);
    println!("trailer dict {:?}", dict);
    // read Info obj
    // read Root obj
    Ok(0)
}

// obj ...  endobj
fn read_object(buf_reader: &mut BufReader<File>) {
    // buf_reader.read_until(b'\n', &mut buf);
    let delim = [b'\n', b'\r'];
    if let Ok(line) = read_until(buf_reader, &delim) {
        println!("{}", line);
        //read until endobj
        if let Token::DICT_BEGIN = read_token(buf_reader) {
            let dict = read_dictonary(buf_reader);
            println!("xx {:?}", dict);
            let end_line = read_token(buf_reader);
            println!("should get endobj:{:?}", end_line);
        // panic!("????");
        } else {
        }
    }
}

fn read_until(buf_reader: &mut BufReader<File>, delim: &[u8]) -> io::Result<String> {
    let mut buf: [u8; 1] = [0];

    let mut vec_buf = Vec::new();
    loop {
        if let Ok(1) = buf_reader.read(&mut buf) {
            let ch = buf[0];
            for v in delim {
                println!("{}  >>> {} ",ch, *v);
                if ch == *v {
                    let line = String::from_utf8(vec_buf).unwrap();
                    return Ok(line);
                }
            }
            vec_buf.push(ch);
        } else {
            return Err(Error::new(ErrorKind::Other, "read error"));
        }
    }
}

/*
<</Size 29
/Root 3 0 R
/Info 1 0 R>>
*/
fn read_dictonary(buf_reader: &mut BufReader<File>) -> io::Result<Dict> {
    let mut dict = Dict::new();
    println!("read_dict");
    let mut check_dict_begin = false;
    loop {
        let tk = read_token(buf_reader);
        println!("{:?}", tk);
        if !check_dict_begin {
            check_dict_begin = true;
            if let Token::DICT_BEGIN = tk {
                continue;
            }
        }
        if let Token::DICT_END = tk {
            break;
        }
        if let Token::NAME(key) = tk {
            //read value
            let tk = read_token(buf_reader);
            match tk {
                Token::INTEGER(n) => {
                    // 偷窥下一个token
                    let tk2 = peek_token(buf_reader);
                    match tk2 {
                        Token::INTEGER(n2) => {
                            // shoule is REF, we read next Token "R"
                            let tk = read_token(buf_reader); // peek to read
                            if let Token::R = read_token(buf_reader) {
                                dict.push(key, Value::REF(n, n2));
                            } else {
                                //error
                            }
                        }
                        _ => {
                            dict.push(key, Value::INTEGER(n));
                        }
                    }
                }
                Token::BOOL(b) => {
                    dict.push(key, Value::BOOL(b));
                }
                Token::STRING(s) => {
                    dict.push(key, Value::STRING(s));
                }
                Token::ARRAY_BEGIN => {
                    let value = read_array(buf_reader);
                    dict.push(key, Value::ARRAY(value));
                }
                Token::DICT_BEGIN => {
                    if let Ok(value) = read_dictonary(buf_reader) {
                        dict.push(key, Value::DICT(value));
                    } else {
                        // TODO
                    }
                }
                Token::NULL => {
                    dict.push(key, Value::NULL);
                }
                Token::FLOAT(v) => {
                    dict.push(key, Value::FLOAT(v));
                }
                Token::NAME(v) => {
                    dict.push(key, Value::NAME(v));
                }
                _ => {}
            }
        } else {
            println!("error happen {:?}", tk);
        }
    }
    Ok(dict)
}

fn read_array(buf_reader: &mut BufReader<File>) -> Vec<Value> {
    // TODO
    let mut array: Vec<Value> = Vec::new();
    loop {
        let tk = read_token(buf_reader);
        match tk {
            Token::ARRAY_END => {
                break;
            }
            Token::INTEGER(v) => {
                array.push(Value::INTEGER(v));
            }
            Token::FLOAT(v) => {
                array.push(Value::FLOAT(v));
            }
            Token::STRING(v) => {
                array.push(Value::STRING(v));
            }
            Token::NAME(v) => {
                array.push(Value::NAME(v));
            }
            Token::BOOL(v) => {
                array.push(Value::BOOL(v));
            }
            Token::NULL => {
                array.push(Value::NULL);
            }
            Token::ARRAY_BEGIN => {
                let val = read_array(buf_reader);
                array.push(Value::ARRAY(val));
            }

            Token::ERROR(e) => {
                //TODO
                break;
            }
            _ => {
                panic!("should no be here");
            }
        }
    }
    array
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
