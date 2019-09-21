use rpdf::Pdf;
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let path = &args[1];
        let pdf = Pdf::open(path.as_str());
        println!("{:?}", pdf);
    } else {
        let pdf = Pdf::open("readme.pdf");
        println!("{:?}", pdf);
    }
}
