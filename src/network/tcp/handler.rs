extern crate mio;

use self::mio::net::TcpStream;
use helpers::{get_random_token_from_map, Log};
use mio::{Events, Poll, Token, Waker};
use network::server::{ServerConnectionEventCallback, ServerConnectionEvents};
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
    event_sender_channel: Sender<(ServerConnectionEvents, ServerConnectionEventCallback)>,
    #[allow(dead_code)]
    handler_thread: JoinHandle<()>,
}

pub struct TcpHandler {
    poll: Poll,
    socket_receiver_channel: Receiver<TcpStream>,
    event_receiver_channel: Receiver<(ServerConnectionEvents, ServerConnectionEventCallback)>,
    #[allow(dead_code)]
    poll_waker: Arc<Waker>,
    connections: HashMap<Token, TcpConnection>,
    event_callbacks: HashMap<ServerConnectionEvents, Vec<ServerConnectionEventCallback>>,
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
        let (event_sender, event_receiver) =
            mpsc::channel::<(ServerConnectionEvents, ServerConnectionEventCallback)>();
        TcpHandlerSender {
            poll_waker: waker.clone(),
            socket_sender_channel: sender,
            event_sender_channel: event_sender,
            handler_thread: thread::spawn(move || {
                let mut tcp_handler = TcpHandler {
                    poll,
                    socket_receiver_channel: receiver,
                    event_receiver_channel: event_receiver,
                    poll_waker: waker,
                    connections: HashMap::new(),
                    event_callbacks: HashMap::new(),
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
                    let got_socket = match self.socket_receiver_channel.try_recv() {
                        Ok(tcp_socket) => {
                            let conn_token = get_random_token_from_map(&self.connections);
                            let mut tcp_conn = TcpConnection::new(tcp_socket, conn_token);
                            if tcp_conn.register(&self.poll) {
                                self.connections.insert(conn_token, tcp_conn);
                            } else {
                                drop(tcp_conn);
                            }
                            true
                        }
                        Err(_e) => false,
                    };
                    if !got_socket {
                        match self.event_receiver_channel.try_recv() {
                            Ok((event, callback)) => self.on(event, callback),
                            Err(_e) => {}
                        };
                    }
                } else if let Some(tcp_conn) = self.connections.get_mut(&event_token) {
                    if event.is_readable() {
                        if tcp_conn.api_version == 0 {
                            tcp_conn.read_api_version();
                        }

                        if let Some(data_buffer) = tcp_conn.read_data() {
                            println!("{:?}", self.event_callbacks.len());
                            if let Some(callbacks) =
                                self.event_callbacks.get(&ServerConnectionEvents::Message)
                            {
                                for callback in callbacks {
                                    let response = callback(&data_buffer);
                                    if response.len() > 0 {
                                        tcp_conn.write(response, &self.poll);
                                    }
                                }
                            }
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

    pub fn on(&mut self, event: ServerConnectionEvents, callback: ServerConnectionEventCallback) {
        if let Some(event_callbacks) = self.event_callbacks.get_mut(&event) {
            event_callbacks.push(callback);
        } else {
            self.event_callbacks.insert(event, vec![callback]);
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

    pub fn send_event(
        &self,
        event: ServerConnectionEvents,
        callback: ServerConnectionEventCallback,
    ) {
        match self.event_sender_channel.send((event, callback)) {
            Ok(()) => (),
            Err(e) => {
                Log::error(
                    "Unable to transfer Event to handler",
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
