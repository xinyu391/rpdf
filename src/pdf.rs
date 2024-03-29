use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use std::io::{Error, ErrorKind};
use std::str;
use std::vec::*;

#[path = "filter.rs"]
mod filter;
use filter::*;

#[path = "parse.rs"]
mod parse;
use parse::*;

const PDF_NAME_Root: &str = "Root";
const PDF_NAME_Type: &str = "Type";
const PDF_NAME_Length: &str = "Length";
const PDF_NAME_FlateDecode: &str = "FlateDecode";
const PDF_NAME_Filter: &str = "Filter";
const PDF_NAME_Pages: &str = "Pages";
const PDF_NAME_Count: &str = "Count";
const PDF_NAME_Kids: &str = "Kids";
const PDF_NAME_MediaBox: &str = "MediaBox";
const PDF_NAME_Contents: &str = "Contents";
const PDF_NAME_Parent: &str = "Parent";
const PDF_NAME_Resources: &str = "Resources";
const PDF_NAME_Font: &str = "Font";

#[derive(Debug)]
struct Obj {
    id: i32,
    offset: i32,
    genid: i32,
    used: bool,
    // box real data
    dict: Option<Dict>,
    stream: Option<Stream>,
}
impl Obj {
    fn new(id: i32, offset: i32, genid: i32, used: bool) -> Obj {
        Obj {
            id,
            offset,
            genid,
            used,
            dict: None,
            stream: None,
        }
    }
    fn fill() -> bool {
        false
    }
    fn get(&self, key: &str) -> Option<&Value> {
        match &self.dict {
            Some(dict) => dict.get(key),
            _ => None,
        }
    }
}
#[derive(Debug)]
pub struct Pdf {
    version: String,
    obj_list: HashMap<i32, Obj>,
    trailer: Option<Dict>,
    root_id: i32,
    pages_id: i32,
    page_count: i32,
}

impl Pdf {
    pub fn new() -> Pdf {
        Pdf {
            version: "".to_string(),
            obj_list: HashMap::new(),
            trailer: None,
            root_id: 0,
            page_count: 0,
            pages_id: 0,
        }
    }
    pub fn open(path: &str) -> io::Result<Pdf> {
        let mut file = File::open(path).unwrap();
        println!("{:?}", file);
        let len: u64 = match file.seek(SeekFrom::End(0)) {
            Ok(len) => len,
            Err(_) => 0,
        };
        println!("file length {}", len);
        file.seek(SeekFrom::Start(0))?;
        let mut buf_reader = BufReader::new(file);
        // let n = buf_reader.read_line(&mut ver);
        let eol = [b'\n', b'\r'];
        if let Ok(ver) = read_until(&mut buf_reader, &eol) {
            println!("ver {}", ver);
            if &ver[0..5] == "%PDF-" {
                println!("match {}", &ver[5..]);
            }
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
            // println!("{}", buffer);
        }
        println!("ref_start_oos {}", ref_offset);
        if ref_offset == 0 {
            return Err(Error::new(ErrorKind::Other, "ref_offset"));
        }
        // Err("?")
        buf_reader.seek(SeekFrom::Start(ref_offset as u64))?;
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
                pdf.trailer = read_trailer(&mut buf_reader);
                break;
            }
            let iter: Vec<i32> = buffer
                .split_whitespace()
                .map(|x| x.parse::<i32>().unwrap())
                .collect();
            let mut oid = iter[0]; //.parse().unwrap();
            let count = iter[1]; //.parse().unwrap();
            for i in 0..count {
                buffer.clear();
                buf_reader.read_line(&mut buffer)?;
                let three: Vec<&str> = buffer.split_whitespace().collect();
                println!("{:?}", three);
                let offset: i32 = three[0].parse().expect("??");
                let genid: i32 = three[1].parse().expect("??");
                let sign: bool = match three[2] {
                    "n" => true,
                    _ => false,
                };
                let obj = Obj::new(oid, offset, genid, sign);
                pdf.obj_list.insert(oid, obj);
                oid += 1;
            }
            // println!("{:?}", iter);
        }
        // read all obj
        read_objects(&mut pdf, &mut buf_reader);
        if pdf.root_id > 0 {
            // load_doc
            load_doc(&mut pdf);
            return Ok(pdf);
        }
        return Err(Error::new(ErrorKind::Other, "not found root"));
    }
    pub fn page_count(&self) -> i32 {
        self.page_count
    }
    pub fn page_text(&self, no: i32) -> String {
        load_page(self, no);
        String::from("page?")
    }
}
fn load_doc(pdf: &mut Pdf) {
    if let Some(root) = pdf.obj_list.get(&pdf.root_id) {
        if let Some(Value::REF(n0, _)) = root.get(PDF_NAME_Pages) {
            pdf.pages_id = *n0;
            if let Some(pages) = pdf.obj_list.get(n0) {
                if let Some(Value::INTEGER(n)) = pages.get(PDF_NAME_Count) {
                    println!("page count is {}", n);
                    pdf.page_count = *n;
                }
                if let Some(Value::ARRAY(kids)) = pages.get(PDF_NAME_Kids) {
                    println!("kids {:?}", kids);
                }
            }
        }
    }
}
fn load_page(pdf: &Pdf, pn: i32) {
    if let Some(pages) = pdf.obj_list.get(&pdf.pages_id) {
        if let Some(Value::ARRAY(kids)) = pages.get(PDF_NAME_Kids) {
            println!("kids {:?}", kids);
            if let Some(Value::REF(n0, n1)) = kids.get(pn as usize) {
                if let Some(page) = pdf.obj_list.get(&n0) {
                    if let Some(Value::ARRAY(rect)) = page.get(PDF_NAME_MediaBox) {
                        println!("page Size {:?}", rect);
                    }
                    match page.get(PDF_NAME_Contents) {
                        Some(Value::STRING(s)) => {
                            println!("page content {}", s);
                        }
                        Some(Value::REF(n0, n1)) => {
                            if let Some(content) = pdf.obj_list.get(&n0) {
                                if let Some(stream) = &content.stream {
                                    if let Ok(data) = str::from_utf8(&stream.data) {
                                        // let content = String::from_utf8(data);
                                        println!("content stream is {}", data);
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

fn read_objects(pdf: &mut Pdf, buf_reader: &mut BufReader<File>) {
    let mut has_root = false; // TOOD pdf.trailer.
    if let Some(trailer) = &pdf.trailer {
        if let Some(Value::REF(n0, n1)) = trailer.get(PDF_NAME_Root) {
            pdf.root_id = *n0;
            has_root = true;
        }
    }
    // for obj in &mut pdf.obj_list {
    for (key, obj) in &mut pdf.obj_list {
        println!("{:?}", obj);
        if obj.used {
            if let Ok(_) = buf_reader.seek(SeekFrom::Start(obj.offset as u64)) {
                read_object(buf_reader, obj);
                if !has_root {
                    if let Some(dict) = &obj.dict {
                        // try find root
                        if let Some(Value::REF(n0, n1)) = dict.get(PDF_NAME_Root) {
                            // record root id TODO
                            pdf.root_id = *n0;
                            has_root = true;
                        }
                    }
                }
            }
        }
    }
}
fn read_trailer(buf_reader: &mut BufReader<File>) -> Option<Dict> {
    if let Ok(dict) = read_dictonary(buf_reader) {
        println!("trailer dict {:?}", dict);
        // read Info obj
        // read Root obj
        return Some(dict);
    }
    None
}

// obj ...  endobj
fn read_object(buf_reader: &mut BufReader<File>, obj: &mut Obj) {
    if let Token::INTEGER(oid) = read_token(buf_reader) {
        if let Token::INTEGER(gid) = read_token(buf_reader) {
            if let Token::OBJ_BEGIN = read_token(buf_reader) {
                println!("{} {} obj", oid, gid);
                if let Token::DICT_BEGIN = read_token(buf_reader) {
                    if let Ok(dict) = read_dictonary(buf_reader) {
                        match read_token(buf_reader) {
                            Token::OBJ_END => {}
                            Token::STREAM_BEGIN => {
                                if let Some(val) = dict.get(PDF_NAME_Length) {
                                    if let Value::INTEGER(len) = val {
                                        if let Ok(mut stream) =
                                            read_stream(buf_reader, *len as usize)
                                        {
                                            // decode stream
                                            println!("len {}", len);
                                            let fs = String::from(PDF_NAME_FlateDecode);
                                            match dict.get(PDF_NAME_Filter) {
                                                Some(Value::NAME(fs)) => {
                                                    if let Ok(mut decoded) =
                                                        decode_deflate(&stream.data[..], fs)
                                                    {
                                                        // let s = String::from_utf8(decoded);
                                                        // println!("inflatexxxx xxx {:?}",s);
                                                        // panic!("xx");
                                                        stream.data.clear();
                                                        stream.data.append(&mut decoded);
                                                    }
                                                }
                                                Some(Value::ARRAY(array)) => {
                                                    panic!("xxxxxxxxx");
                                                }
                                                Some(n) => {
                                                    println!("xxxxxxxx {:?}", n);
                                                    panic!("xxxxxxxxx ");
                                                }
                                                None => (),
                                                _ => {
                                                    panic!("xxxxxxxxx");
                                                }
                                            }
                                            // if let Some(Value::NAME(fs)) = dict.get("Filter"){
                                            // }
                                            obj.stream = Some(stream);
                                        } else {
                                            println!("what's wrong?");
                                        }
                                    }
                                }
                            }
                            _ => {
                                println!("should get endobj:");
                            }
                        }
                        obj.dict = Some(dict);
                    }
                    // panic!("????");
                }
            }
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
                // println!("{}  >>> {} ",ch, *v);
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
    println!("read_array");
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
            Token::R => {
                if let Some(Value::INTEGER(n1)) = array.pop() {
                    if let Some(Value::INTEGER(n0)) = array.pop() {
                        array.push(Value::REF(n0, n1));
                    } else {
                        // error
                        panic!("not a REF");
                    }
                } else {
                    // error
                    panic!("not a REF");
                }
            }
            Token::ERROR(e) => {
                //TODO
                break;
            }
            Token::DICT_BEGIN => {
                if let Ok(value) = read_dictonary(buf_reader) {
                    array.push(Value::DICT(value));
                } else {
                    // error
                    panic!("not a DICT");
                }
            }
            _ => {
                panic!("should no be here {:?}", tk);
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
