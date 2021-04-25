#[macro_use]
extern crate lazy_static;

mod model;
mod protocol;
mod worker;
use uuid::Uuid;

fn main() {
    let id = Uuid::new_v4();
    println!("{:?}", id);
}