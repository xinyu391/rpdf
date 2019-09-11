
use rpdf::Pdf;

fn main() {
    println!("Hello, world!");
    let pdf = Pdf::open("readme.pdf");
    println!("{:?}",pdf);
}
