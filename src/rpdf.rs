use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use std::io::{Error, ErrorKind};
use std::str;

#[path ="parse.rs"]
mod parse;
use parse::*;

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

/*
<</Size 29
/Root 3 0 R
/Info 1 0 R>>
*/
fn read_dictonary(buf_reader: &mut BufReader<File>) -> io::Result<usize> {
    let mut dict = Dict::new();
    loop{
        let tk =  read_token(buf_reader);
        println!("read_dict {:?}",tk);
        // match tk{
        //     Token::DICT_END =>
        // }
    }
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
