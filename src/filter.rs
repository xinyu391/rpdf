
use std::io;
use std::io::{Error, ErrorKind};

extern crate inflate;
pub fn decode_deflate(data :&[u8], name :&String)->io::Result<Vec<u8>>{
    if name == "FlateDecode"{
        // if let Ok(d) = inflate::inflate_bytes_zlib(data){
        //     return Ok(d);
        // }
        return inflate::inflate_bytes_zlib(data).map_err(|err|Error::new(ErrorKind::Other,""));
    }
    Err(Error::new(ErrorKind::Other,"not support decode name"))
    // if let Ok(mut decoded) = inflate::inflate_bytes_zlib(&stream.data[..]){
// 
    // }
}