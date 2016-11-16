#![allow(dead_code)]
#![allow(unreachable_code)]
extern crate mio;

use network::tcp::TcpReaderConn;
use std::io::Result;
use self::mio::{Poll, Token, Ready, PollOpt, Events};
use self::mio::channel::{Receiver, Sender, channel};
use self::mio::tcp::TcpStream;
use std::sync::Arc;

/// Read buffer size 64KB
const READER_READ_BUFFER_SIZE: usize = 65000;
const READER_CHANNEL_TOKEN: Token = Token(1);

pub enum TcpReaderCMD {
    HandleNewConnection,
    CloseConnection,
    SendData,
}

pub struct TcpReaderCommand {
    // base command code
    code: TcpReaderCMD,
    // socket vector for transfering new connection
    socket: Vec<TcpStream>,
    token: Vec<Token>,
    data: Vec<Arc<Vec<u8>>>
}

pub struct TcpReader {
    // connections transferred to this reader for IO operations
    connections: Vec<TcpReaderConn>,

    // buffers for making one time allocations per read process
    data_len_buf: Vec<u8>,
    data_chunk: Vec<u8>,

    // base event loop handler
    poll: Poll,

    // chanel sender, receiver for keeping communication with loop
    channel_sender: Sender<TcpReaderCommand>,
    channel_receiver: Receiver<TcpReaderCommand>
}

impl TcpReader {
    /// creating new TcpReader with default values
    pub fn new() -> TcpReader {
        let (s, r)= channel::<TcpReaderCommand>();
        TcpReader {
            connections: Vec::new(),
            data_len_buf: vec![0; 4],
            data_chunk: vec![0; READER_READ_BUFFER_SIZE],
            poll: Poll::new().unwrap(),
            channel_sender: s,
            channel_receiver: r
        }
    }

    /// Clonning channel for sending commands
    pub fn channel(&self) -> Sender<TcpReaderCommand> {
        self.channel_sender.clone()
    }

    /// Private function for handling Reader commands
    #[inline(always)]
    fn notify(&mut self, cmd: &mut TcpReaderCommand) {
        match cmd.code {
            TcpReaderCMD::HandleNewConnection => {
                // Handling new connection with given socket
                // if it exists in Vector of sockets
                while !cmd.socket.is_empty() && !cmd.token.is_empty() {
                    let sock = match cmd.socket.pop() {
                        Some(sock) => sock,
                        None => return
                    };

                    let token = match cmd.token.pop() {
                        Some(t) => t,
                        None => return
                    };

                    self.connections.push(TcpReaderConn::new(sock, token));
                }
            }

            TcpReaderCMD::CloseConnection => {
                // Closing connection by given token
                while !cmd.token.is_empty() {
                    let token = match cmd.token.pop() {
                        Some(t) => t,
                        _ => return
                    };

                    // if we have this connection
                    // just removing it from our list
                    // after removing it will be automatically deatached from loop
                    for i in 0..self.connections.len() {
                        if self.connections[i].token == token {
                            self.connections.remove(i);
                            break;
                        }
                    }
                }
            }

            TcpReaderCMD::SendData => {
                // if data is empty just returning
                if cmd.data.len() == 0 {
                    return;
                }

                // Closing connection by given token
                while !cmd.token.is_empty() {
                    let token = match cmd.token.pop() {
                        Some(t) => t,
                        _ => return
                    };

                    // if we have this connection
                    // adding sent data to our queue for writing
                    // and making connection writable
                    for i in 0..self.connections.len() {
                        if self.connections[i].token == token {
                            self.connections[i].write_queue.append(&mut cmd.data);
                            self.make_writable(&self.connections[i]);
                            break;
                        }
                    }
                }
            }
        }
    }

    /// running TcpReader loop
    /// this will exit when loop is no longer running
    pub fn run(&mut self) -> Result<()> {
        // registering receiver for poll loop
        match self.poll.register(&self.channel_receiver, READER_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }

        let mut events: Events = Events::with_capacity(1000);

        loop {
            // using unwrap here because if it is failing anyway process should be closed
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue
            }

            for event in events.into_iter() {
                let token = event.token();
                if token == READER_CHANNEL_TOKEN {
                    match self.channel_receiver.try_recv() {
                        Ok(cmd) => {
                            let mut c = cmd;
                            self.notify(&mut c);
                        }
                        Err(_) => {}
                    }
                }
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn make_writable(&self, conn: &TcpReaderConn) {
        let mut r = Ready::readable();
        r.insert(Ready::writable());
        let _ = self.poll.reregister(
            &conn.socket, conn.token, r,
            PollOpt::edge() | PollOpt::oneshot()
        );
    }
}
