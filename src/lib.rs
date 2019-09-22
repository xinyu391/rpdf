mod pdf;
pub use self::pdf::*;
use std::io;

pub fn open(path:&str)->io::Result<Pdf>{
    Pdf::open(path)
}