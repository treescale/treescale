extern crate mio;

use self::mio::net::TcpListener;
use helpers::Log;
use mio::{Events, Interest, Poll, Token};
use network::server::{ServerConnectionEventCallback, ServerConnectionEvents};
use network::tcp::handler::{TcpHandler, TcpHandlerSender};
use std::net::SocketAddr;
use std::process;

const SERVER_TOKEN: Token = Token(0);
const SERVER_CAPACITY: usize = 256;

pub struct TcpServer {
    server: TcpListener,
    poll: Poll,
    client_handlers: Vec<TcpHandlerSender>,
}

impl TcpServer {
    pub fn new(address: &str, concurrency: usize) -> TcpServer {
        let parsed_address = address.parse::<SocketAddr>().unwrap_or_else(|e| {
            panic!(
                "Unable to parse given {} network address -> {}",
                address,
                e.to_string().as_str()
            )
        });

        let mut client_handlers: Vec<TcpHandlerSender> = Vec::with_capacity(concurrency);
        for _ in 0..concurrency {
            client_handlers.push(TcpHandler::start());
        }

        TcpServer {
            server: match TcpListener::bind(parsed_address) {
                Ok(s) => s,
                Err(e) => {
                    Log::error("Unable to bind to an address", e.to_string().as_str());
                    process::exit(1);
                }
            },
            poll: Poll::new().expect("Unable to start OS Poll"),
            client_handlers,
        }
    }

    pub fn listen(&mut self) {
        match self
            .poll
            .registry()
            .register(&mut self.server, SERVER_TOKEN, Interest::READABLE)
        {
            Ok(r) => r,
            Err(e) => {
                Log::error(
                    "Unable to register bound server for the events",
                    e.to_string().as_str(),
                );
                process::exit(1);
            }
        };

        // Create storage for events.
        let mut events = Events::with_capacity(SERVER_CAPACITY);
        let mut client_handler_index = 0;

        loop {
            match self.poll.poll(&mut events, None) {
                Ok(()) => (),
                Err(e) => {
                    Log::error(
                        "Unable to get TcpServer events from Poll",
                        e.to_string().as_str(),
                    );
                    continue;
                }
            };
            for event in events.iter() {
                if event.token() != SERVER_TOKEN {
                    continue;
                }

                let (tcp_stream, _) = match self.server.accept() {
                    Ok(c) => c,
                    Err(e) => {
                        Log::error(
                            "Unable to accept client connection in TcpServer",
                            e.to_string().as_str(),
                        );
                        continue;
                    }
                };
                if client_handler_index >= self.client_handlers.len() {
                    client_handler_index = 0;
                }
                self.client_handlers[client_handler_index].send_socket(tcp_stream);
                client_handler_index += 1;
            }
        }
    }

    pub fn on(&mut self, event: ServerConnectionEvents, callback: ServerConnectionEventCallback) {
        for handler in self.client_handlers.as_slice() {
            handler.send_event(event.clone(), callback.clone());
        }
    }
}
