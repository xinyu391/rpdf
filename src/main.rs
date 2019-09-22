mod pdf;
mod parse;

use rpdf;

use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let path = &args[1];
        let pdf = rpdf::open(path.as_str());
        println!("{:?}", pdf);
    } else {
        let pdf = rpdf::open("readme.pdf");
        println!("{:?}", pdf);
    }
}
