mod parse;

use rpdf;

use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();

    let mut path = "readme.pdf";

    if args.len() > 1 {
        path = &args[1].as_str();
    }
    if let Ok(pdf) = rpdf::open(path) {
        println!("{:?}", pdf);
        println!("page count: {:?}", pdf.page_count());
        pdf.page_text(0);
    }
}
