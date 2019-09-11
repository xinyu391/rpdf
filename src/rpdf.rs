use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::str;
use std::io;

#[derive(Debug)]
pub struct Pdf{
    version : String,
}

impl Pdf{
   pub  fn open(path :&'static str) -> io::Result<Pdf>{
        let mut file = File::open(path).unwrap();
        println!("{:?}",file);
        let len:u64 = match file.seek(SeekFrom::End(0)){
            Ok(len) => len,
            Err(_) => 0,
        };
        println!("file length {}",len);
        file.seek(SeekFrom::Start(0));
        let mut buf_reader = BufReader::new(file);
        let mut buf: Vec<u8>  = Vec::with_capacity(10);
        buf_reader.read_until(b'\n',&mut buf);
        
        let len = buf.len()-1;
        buf[len]=b'\0';
        let ver = String::from_utf8(buf).unwrap();//str::from_utf8(&buf).unwrap();
        println!("version {}. {}",ver, ver.len());
        if &ver[0..5] == "%PDF-"{
            println!("match {}", &ver[5..] );
        }
        // startxref
        buf_reader.seek(SeekFrom::End(-32));

        // read tails
        let mut ref_offset: usize = 0 ;
        for i in 0..4{
            let mut buffer = String::new();
            buf_reader.read_line(&mut buffer)?;
            if buffer == "startxref\n"{
                 let mut buffer = String::new();
                  buf_reader.read_line(&mut buffer)?;
                  ref_offset = buffer.trim().parse().expect("??");
                  break;
            }
              println!("{}", buffer);
        }
        println!("ref_start_oos {}", ref_offset);
        if ref_offset ==0{
             return Err(Error::new(ErrorKind::Other,"ref_offset"));
        }
           // Err("?")
            buf_reader.seek(SeekFrom::Start(ref_offset as u64));
            let mut buffer = String::new();
            buf_reader.read_line(&mut buffer)?;
            if buffer !="xref\n"{
               return Err(Error::new(ErrorKind::Other,"ref sign"));
            }
            loop{
                let mut buffer = String::new();
                buf_reader.read_line(&mut buffer)?;
                 if buffer == "trailer\n"{
                    break;
                }
                let  iter: Vec<&str>= buffer.split_whitespace().collect();

                let count:u32 =  iter[1].parse().unwrap();
                for i in 0..count{
                    buffer.clear();
                     buf_reader.read_line(&mut buffer)?;
                     let  three: Vec<&str>= buffer.split_whitespace().collect();
                     println!("{:?}", three);
                }
                // println!("{:?}", iter);
            }
        
        Ok(Pdf{version:ver})
    }
}