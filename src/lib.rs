mod pdf;
pub use self::pdf::*;
use std::io;

pub fn open(path:&str)->io::Result<Pdf>{
    Pdf::open(path)
}
pub fn open_with_pwd(path:&str, pwd:&str)->io::Result<Pdf>{
    // Pdf::open_with_pwd(path) // TODO
    Pdf::open(path)
}