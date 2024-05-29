// https://users.rust-lang.org/t/is-there-a-way-to-put-an-image-on-your-executable/62996/6

fn main() {
    println!("cargo:rustc-link-lib=./resources/res");
}