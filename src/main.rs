extern crate core;
extern crate mio;
extern crate rand;

mod constants;
mod helpers;
mod network;

use std::env;
use std::sync::Arc;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 || args[1] == "server" {
        let mut server = network::Server::new("127.0.0.1:5000", 5);
        server.on_message(Arc::new(move |data, connection| {
            println!(
                "ADDRESS: {} -> {}",
                connection
                    .remote_address()
                    .expect("No Socket Address")
                    .to_string(),
                String::from_utf8(data.clone()).expect("Not a UTF-8 string")
            );
            connection.write(data.clone());
        }));
        server.listen();
    } else if args[1] == "client" {
        let mut client = network::TcpClient::new("127.0.0.1:5000", 5);
        client.on_message(|data| {
            println!(
                "{}",
                String::from_utf8(data.clone()).expect("Not a UTF-8 string")
            );
            Vec::from("Test")
        });
        client.send(Vec::from("Test"));
        client.start();
    }
}
