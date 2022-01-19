extern crate mio;

use std::net::SocketAddr;
use self::mio::net::{ TcpListener };
use std::process;
use mio::{Poll, Token, Interest, Events};
use helpers::Log;
use network::tcp::handler::TcpHandler;

const SERVER_TOKEN: Token = Token(0);
const SERVER_CAPACITY: usize = 256;

pub struct TcpServer {
    server: TcpListener,
    poll: Poll,
}

impl TcpServer {
    pub fn new(address: &str) -> TcpServer {
        let parsed_address = match address.parse::<SocketAddr>() {
            Ok(p) => p,
            Err(e) => {
                Log::error("Unable to parse given server address", e.to_string().as_str());
                process::exit(1);
            }
        };

        TcpServer {
            server: match TcpListener::bind(parsed_address) {
                Ok(s) => s,
                Err(e) => {
                    Log::error("Unable to bind to an address", e.to_string().as_str());
                    process::exit(1);
                }
            },
            poll: match Poll::new() {
                Ok(p) => p,
                Err(e) => {
                    Log::error("Unable to start an OS Poll", e.to_string().as_str());
                    process::exit(1);
                }
            }
        }
    }

    pub fn listen(&mut self) {
        match self.poll.registry().register(&mut self.server, SERVER_TOKEN, Interest::READABLE) {
            Ok(r) => r,
            Err(e) => {
                Log::error("Unable to register bound server for the events", e.to_string().as_str());
                process::exit(1);
            }
        };

        // Create storage for events.
        let mut events = Events::with_capacity(SERVER_CAPACITY);

        let tcp_handler_sender = TcpHandler::start();

        loop {
            match self.poll.poll(&mut events, None) {
                Ok(()) => (),
                Err(e) => {
                    Log::error("Unable to get TcpServer events from Poll", e.to_string().as_str());
                    continue
                }
            };
            for event in events.iter() {
                if event.token() != SERVER_TOKEN {
                    continue
                }

                let (tcp_stream, _) = match self.server.accept() {
                    Ok(c) => c,
                    Err(e) => {
                        Log::error("Unable to accept client connection in TcpServer", e.to_string().as_str());
                        continue
                    }
                };
                Log::info("Got new Client Event", "DROPPING!");
                tcp_handler_sender.send_socket(tcp_stream)
            }
        }
    }
}
