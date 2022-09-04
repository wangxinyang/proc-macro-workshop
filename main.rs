// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use derive_builder::Builder;

#[warn(dead_code)]
#[derive(Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
}

// #[derive(Builder)]
// pub enum Person {
//     Male = 1,
//     Female = 2,
// }

fn main() {
    let mut builder = Command::builder();
    println!("{:#?}", builder.executable("hahah".to_string()));
}
