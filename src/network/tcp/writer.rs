#![allow(dead_code)]
extern crate mio;
extern crate slab;

use self::mio::channel::{channel, Sender, Receiver};
use self::mio::{Poll, Ready, PollOpt, Token, Events};
use network::tcp::{TcpNetworkCommand, TcpWriterConn};
use network::{NetworkCommand};
use node::{NodeCommand};
use std::process;
use std::u32::MAX as u32MAX;
use std::sync::Arc;

type Slab<T> = slab::Slab<T, Token>;
const RECEIVER_CHANNEL_TOKEN: Token = Token(u32MAX as usize);

pub struct TcpWriter {
    // TcpNetworking channel for sending commands to it
    pub tcp_net_channel: Sender<TcpNetworkCommand>,

    // Channel to base Networking for passing commands to it
    pub network_channel: Sender<NetworkCommand>,
    node_channel: Sender<NodeCommand>,

    // Sender and Receiver for handling commands for TcpReader
    sender_channel: Sender<TcpWriterCommand>,
    receiver_channel: Receiver<TcpWriterCommand>,

    // POLL service for current Reader service
    poll: Poll,

    // List of connections handled by this reader service
    connections: Slab<TcpWriterConn>
}

pub enum TcpWriterCMD {
    HandleNewConnection,
    WriteData,
}

pub struct TcpWriterCommand {
    pub cmd: TcpWriterCMD,
    pub conn: Vec<TcpWriterConn>,
    pub token: Vec<Token>,
    pub data: Vec<Arc<Vec<u8>>>
}

impl TcpWriter {
    pub fn new(tcp_net: Sender<TcpNetworkCommand>
        , net: Sender<NetworkCommand>, node_chan: Sender<NodeCommand>) -> TcpWriter {
        let (s, r) = channel::<TcpWriterCommand>();
        TcpWriter {
            tcp_net_channel: tcp_net,
            network_channel: net,
            sender_channel: s,
            receiver_channel: r,
            poll: Poll::new().expect("Unable to create Poll Service for TcpReader"),
            connections: Slab::with_capacity(1024),
            node_channel: node_chan
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpWriterCommand> {
        self.sender_channel.clone()
    }

    pub fn start(&mut self) {
        match self.poll.register(&self.receiver_channel, RECEIVER_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                warn!("Error while trying to register Receiver Channel for TCP Reader -> {}", e);
                process::exit(1);
            }
        };

        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(5000);
        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.iter() {
                let token = event.token();
                if token == RECEIVER_CHANNEL_TOKEN {
                    // trying to get commands while there is available data
                    loop {
                        match self.receiver_channel.try_recv() {
                            Ok(cmd) => {
                                let mut c = cmd;
                                self.notify(&mut c);
                            }
                            // if we got error, then data is unavailable
                            // and breaking receive loop
                            Err(_) => break
                        }
                    }
                    continue;
                }

                let kind = event.kind();

                if kind.is_writable() {
                    self.writable(token);
                } else if kind.is_error() || kind.is_hup() {
                    // if this error on connection, then we need to close it
                    self.close_connection(token);
                }
            }
        }
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut TcpWriterCommand) {
        match command.cmd {
            TcpWriterCMD::HandleNewConnection => {
                if command.conn.len() == 0 {
                    return;
                }

                let mut conn = command.conn.remove(0);
                if self.connections.vacant_entry().is_none() {
                    let conns_len = self.connections.len();
                    self.connections.reserve_exact(conns_len);
                }

                let entry = self.connections.vacant_entry().unwrap();
                conn.socket_token = entry.index();
                // if we are unable to register connection to this poll service
                // then just moving to the next connection, by just closing this one
                if !conn.make_writable(&self.poll) {
                    drop(conn);
                    return;
                }

                entry.insert(conn);
            }

            TcpWriterCMD::WriteData => {
                if command.token.len() != 1 {
                    return;
                }

                let token = command.token.remove(0);
                if !self.connections.contains(token) {
                    return;
                }

                let ref mut conn = self.connections[token];
                conn.write_queue.append(&mut command.data);
                // making connection writable
                conn.make_writable(&self.poll);
            }
        }
    }

    #[inline(always)]
    fn writable(&mut self, token: Token) {
        if !self.connections.contains(token) {
            return;
        }

        let res = {
            let ref mut conn = self.connections[token];
            conn.flush_write_queue()
        };
        // if we got None then there was error with socket
        // so closing connection from writer part
        if res.is_none() {
            self.close_connection(token);
        }

        // if we sent all available data, just deregistering connection
        // if we will have more data to write we will just register back connection again
        if res.unwrap() {
            let ref mut conn = self.connections[token];
            let _ = self.poll.deregister(&conn.socket);
        }
    }

    #[inline(always)]
    fn close_connection(&mut self, token: Token) {
        if !self.connections.contains(token) {
            return;
        }

        let conn = self.connections.remove(token).unwrap();
        let _ = self.poll.deregister(&conn.socket);
        // clearing connection memory
        // which will actionally close other socket things
        drop(conn);
    }
}
