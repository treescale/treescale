extern crate mio;

use self::mio::net::TcpStream;
use helpers::{get_random_token_from_map, Log};
use mio::{Events, Poll, Token, Waker};
use network::tcp::connection::TcpConnection;
use std::collections::HashMap;
use std::process;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

const WAKER_TOKEN: Token = Token(0);

pub struct TcpHandlerSender {
    poll_waker: Arc<Waker>,
    socket_sender_channel: Sender<TcpStream>,
    #[allow(dead_code)]
    handler_thread: JoinHandle<()>,
}

pub struct TcpHandler {
    poll: Poll,
    socket_receiver_channel: Receiver<TcpStream>,
    #[allow(dead_code)]
    poll_waker: Arc<Waker>,
    connections: HashMap<Token, TcpConnection>,
}

impl TcpHandler {
    pub fn start() -> TcpHandlerSender {
        let poll = match Poll::new() {
            Ok(p) => p,
            Err(e) => {
                Log::error(
                    "Unable to start TcpHandler Poll for events",
                    e.to_string().as_str(),
                );
                process::exit(1);
            }
        };
        let waker = Arc::new(match Waker::new(poll.registry(), WAKER_TOKEN) {
            Ok(w) => w,
            Err(e) => {
                Log::error(
                    "Unable to register Waker for TcpHandler",
                    e.to_string().as_str(),
                );
                process::exit(1);
            }
        });
        let (sender, receiver) = mpsc::channel::<TcpStream>();
        TcpHandlerSender {
            poll_waker: waker.clone(),
            socket_sender_channel: sender,
            handler_thread: thread::spawn(move || {
                let mut tcp_handler = TcpHandler {
                    poll,
                    socket_receiver_channel: receiver,
                    poll_waker: waker,
                    connections: HashMap::new(),
                };
                tcp_handler.handle_poll()
            }),
        }
    }

    fn handle_poll(&mut self) {
        let mut events = Events::with_capacity(2);
        loop {
            match self.poll.poll(&mut events, None) {
                Ok(()) => (),
                Err(e) => {
                    Log::error(
                        "Unable to handle events for TcpHandler",
                        e.to_string().as_str(),
                    );
                    continue;
                }
            }

            for event in events.iter() {
                let event_token = event.token();
                if event_token == WAKER_TOKEN {
                    let tcp_socket = match self.socket_receiver_channel.recv() {
                        Ok(s) => s,
                        Err(e) => {
                            Log::error(
                                "Unable to get TCP Socket from TcpHandler",
                                e.to_string().as_str(),
                            );
                            continue;
                        }
                    };
                    let conn_token = get_random_token_from_map(&self.connections);
                    let mut tcp_conn = TcpConnection::new(tcp_socket, conn_token);
                    if tcp_conn.register(&self.poll) {
                        self.connections.insert(conn_token, tcp_conn);
                    } else {
                        drop(tcp_conn);
                    }
                } else if let Some(tcp_conn) = self.connections.get_mut(&event_token) {
                    if event.is_readable() {
                        if tcp_conn.api_version > 0 {
                            tcp_conn.read_api_version();
                            println!("API VERSION -> {}", tcp_conn.api_version);
                        } else if let Some(data_buffer) = tcp_conn.read_data() {
                            println!("DATA LENGTH -> {}", data_buffer.len());
                        }
                    } else if event.is_writable() {
                        if let Some(is_done) = tcp_conn.flush_write() {
                            if is_done {
                                tcp_conn.make_readable(&self.poll);
                            }
                        };
                    }
                }
            }
        }
    }
}

impl TcpHandlerSender {
    pub fn send_socket(&self, socket: TcpStream) {
        match self.socket_sender_channel.send(socket) {
            Ok(()) => (),
            Err(e) => {
                Log::error(
                    "Unable to transfer TCP Client socket to TcpHandler",
                    e.to_string().as_str(),
                );
            }
        }

        match self.poll_waker.wake() {
            Ok(()) => (),
            Err(e) => {
                Log::error(
                    "Unable to send Wake message to TcpHandler",
                    e.to_string().as_str(),
                );
            }
        }
    }
}
