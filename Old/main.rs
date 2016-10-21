extern crate mio;

mod error;
mod network;
mod node;

use node::Node;
use std::thread;
use std::time;
use std::env;

fn main() {
    if env::args().nth(1) == Some(String::from("n1")) {
        let mut n = Node::new(false, 1, String::from("2").as_bytes());
        n.run("0.0.0.0:8888");
        n.connect("127.0.0.1:8889");
        thread::sleep(time::Duration::from_secs(5));
        n.write_str(String::from("Test Data").as_bytes(), String::from("5"));
        thread::sleep(time::Duration::from_secs(25));
    } else {
        let mut n = Node::new(false, 1, String::from("2").as_bytes());
        n.run("0.0.0.0:8889");
        // n.connect("127.0.0.1:8888");
        // thread::sleep(time::Duration::from_secs(5));
        // n.write_str(String::from("Test Data").as_bytes(), String::from("5"));
        thread::sleep(time::Duration::from_secs(25));
    }
}
