mod helper;
mod node;
mod event;
mod network;
mod config;

use std::collections::BTreeMap;

fn main() {
    let mut data: BTreeMap<usize, String> = BTreeMap::new();
    for i in 0..10000000 {
        data.insert(i, String::from("sgsdfgsdfgsdfg"));
        data.remove(&i).unwrap();
    }
}