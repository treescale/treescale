use constants::CURRENT_API_VERSION;
use helpers::Log;
use mio::net::TcpStream;
use mio::{Events, Poll, Token};
use network::tcp::connection::TcpConnection;
use std::collections::HashMap;
use std::net::SocketAddr;

const CLIENT_EVENT_CAPACITY: usize = 256;

pub type MessageCallback<'a> = Box<dyn Fn(&Vec<u8>) -> Vec<u8> + 'a>;

pub struct TcpClient<'a> {
    poll: Poll,
    connections: HashMap<Token, TcpConnection>,
    message_callbacks: Vec<MessageCallback<'a>>,
    concurrency: usize,
    next_connection_index: usize,
}

impl<'a> TcpClient<'a> {
    pub fn new(server_host: &str, concurrency: usize) -> TcpClient {
        let parsed_address = server_host.parse::<SocketAddr>().unwrap_or_else(|e| {
            panic!(
                "Unable to parse given {} network address -> {}",
                server_host,
                e.to_string().as_str()
            )
        });
        let mut connections = HashMap::<Token, TcpConnection>::new();
        let poll = Poll::new().expect("Unable to start OS Poll");
        for index in 0..concurrency {
            let tcp_stream = TcpStream::connect(parsed_address).unwrap_or_else(|e| {
                panic!(
                    "Unable to connect to given {} network address -> {}",
                    server_host,
                    e.to_string().as_str()
                )
            });
            let token = Token(index);
            let mut tcp_conn = TcpConnection::new(tcp_stream, token);
            tcp_conn.register(&poll);
            connections.insert(token, tcp_conn);
        }

        TcpClient {
            poll,
            connections,
            message_callbacks: Vec::new(),
            concurrency,
            next_connection_index: 0,
        }
    }

    pub fn on_message<F: Fn(&Vec<u8>) -> Vec<u8> + 'a>(&mut self, callback: F) {
        self.message_callbacks.push(Box::new(callback))
    }

    pub fn send(&mut self, data: Vec<u8>) {
        if self.next_connection_index >= self.concurrency {
            self.next_connection_index = 0;
        }
        self.connections
            .get_mut(&Token(self.next_connection_index))
            .expect("Unable to select connection")
            .write(data, &self.poll);
        self.next_connection_index += 1;
    }

    pub fn start(&mut self) {
        let mut events = Events::with_capacity(CLIENT_EVENT_CAPACITY);
        loop {
            match self.poll.poll(&mut events, None) {
                Ok(()) => (),
                Err(e) => {
                    Log::error(
                        "Unable to get TcpClient events from Poll",
                        e.to_string().as_str(),
                    );
                    continue;
                }
            };

            for event in events.iter() {
                if let Some(tcp_conn) = self.connections.get_mut(&event.token()) {
                    if event.is_readable() {
                        if let Some(data_buffer) = tcp_conn.read_data() {
                            for callback_box in &mut self.message_callbacks {
                                let send_data = callback_box(&data_buffer);
                                if !send_data.is_empty() {
                                    tcp_conn.write(send_data, &self.poll);
                                }
                            }
                        }
                    } else if event.is_writable() {
                        if tcp_conn.api_version == 0 {
                            tcp_conn.write_api_version(CURRENT_API_VERSION);
                        }

                        if let Some(is_done) = tcp_conn.flush_write() {
                            if is_done {
                                tcp_conn.make_readable(&self.poll);
                            }
                        }
                    }
                }
            }
        }
    }
}
