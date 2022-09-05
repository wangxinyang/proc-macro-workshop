// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use derive_builder::Builder;

#[warn(dead_code)]
#[derive(Builder, Debug)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: Option<String>,
}

// #[derive(Builder)]
// pub enum Person {
//     Male = 1,
//     Female = 2,
// }

fn main() {
    let mut builder = Command::builder();
    builder.executable("cargo".to_owned());
    builder.args(vec!["build".to_owned(), "--release".to_owned()]);
    builder.env(vec![]);
    builder.current_dir("..".to_owned());
    // let test = builder.current_dir.clone().unwrap();
    // println!("{:?}", builder);
    let command = builder.build();
    println!("{:?}", command);
}

fn test() -> Result<String, std::boxed::Box<dyn std::error::Error>> {
    let str = "123".clone();
    let err = std::result::Result::Err("hahah".into());
    println!("{}", "hello world");
    println!("err is {:?}", err);
    return err;
}
