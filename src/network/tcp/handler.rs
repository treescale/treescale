extern crate mio;

use std::process;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::thread::{JoinHandle};
use std::thread;
use mio::{Token, Waker, Poll, Events};
use helpers::Log;
use self::mio::net::{ TcpStream };

const WAKER_TOKEN: Token = Token(0);

pub struct TcpHandlerSender {
    poll_waker: Arc<Waker>,
    socket_sender_channel: Sender<TcpStream>,
    #[allow(dead_code)]
    handler_thread: JoinHandle<()>
}

pub struct TcpHandler {
    poll: Poll,
    socket_receiver_channel: Receiver<TcpStream>,
    #[allow(dead_code)]
    poll_waker: Arc<Waker>,
}

impl TcpHandler {
    pub fn start() -> TcpHandlerSender {
        let poll = match Poll::new() {
            Ok(p) => p,
            Err(e) => {
                Log::error("Unable to start TcpHandler Poll for events", e.to_string().as_str());
                process::exit(1);
            }
        };
        let waker = Arc::new(match Waker::new(poll.registry(), WAKER_TOKEN) {
            Ok(w) => w,
            Err(e) => {
                Log::error("Unable to register Waker for TcpHandler", e.to_string().as_str());
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
                    poll_waker: waker
                };
                tcp_handler.handle_poll()
            })
        }
    }

    fn handle_poll(&mut self) {
        let mut events = Events::with_capacity(2);
        loop {
            match self.poll.poll(&mut events, None) {
                Ok(()) => (),
                Err(e) => {
                    Log::error("Unable to handle events for TcpHandler", e.to_string().as_str());
                    continue
                }
            }

            for event in events.iter() {
                if event.token() == WAKER_TOKEN {
                    let tcp_socket = match self.socket_receiver_channel.recv() {
                        Ok(s) => s,
                        Err(e) => {
                            Log::error("Unable to get TCP Socket from TcpHandler", e.to_string().as_str());
                            continue
                        }
                    };
                    Log::info("Just Dropping Tcp Socket", "Tcp Socket");
                    drop(tcp_socket)
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
                Log::error("Unable to transfer TCP Client socket to TcpHandler", e.to_string().as_str());
            }
        }

        match self.poll_waker.wake() {
            Ok(()) => (),
            Err(e) => {
                Log::error("Unable to send Wake message to TcpHandler", e.to_string().as_str());
            }
        }
    }
}