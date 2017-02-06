#![allow(dead_code)]
extern crate mio;
extern crate slab;

use self::mio::channel::{channel, Sender, Receiver};
use self::mio::{Poll, Ready, PollOpt, Token, Events};
use network::tcp::{TcpNetworkCommand, TcpReaderConn};
use network::{NetworkCommand};
use node::{NodeCommand, Event, NodeCMD};
use std::process;
use std::u32::MAX as u32MAX;

type Slab<T> = slab::Slab<T, Token>;
const RECEIVER_CHANNEL_TOKEN: Token = Token(u32MAX as usize);

/// Using this struct we are reading data from TCP connection sockets
pub struct TcpReader {
    // TcpNetworking channel for sending commands to it
    pub tcp_net_channel: Sender<TcpNetworkCommand>,

    // Channel to base Networking for passing commands to it
    pub network_channel: Sender<NetworkCommand>,
    node_channel: Sender<NodeCommand>,

    // Sender and Receiver for handling commands for TcpReader
    sender_channel: Sender<TcpReaderCommand>,
    receiver_channel: Receiver<TcpReaderCommand>,

    // POLL service for current Reader service
    poll: Poll,

    // List of connections handled by this reader service
    connections: Slab<TcpReaderConn>
}

pub enum TcpReaderCMD {
    HandleNewConnection,
}

pub struct TcpReaderCommand {
    pub cmd: TcpReaderCMD,
    pub conn: Vec<TcpReaderConn>
}

impl TcpReader {
    pub fn new(tcp_net: Sender<TcpNetworkCommand>
        , net: Sender<NetworkCommand>, node_chan: Sender<NodeCommand>) -> TcpReader {
        let (s, r) = channel::<TcpReaderCommand>();
        TcpReader {
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
    pub fn channel(&self) -> Sender<TcpReaderCommand> {
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

                if kind.is_readable() {
                    self.readable(token);
                } else if kind.is_error() || kind.is_hup() {
                    // if this error on connection, then we need to close it
                    self.close_connection(token);
                }
            }
        }
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut TcpReaderCommand) {
        match command.cmd {
            TcpReaderCMD::HandleNewConnection => {
                if command.conn.len() == 0 {
                    return;
                }

                let conn = command.conn.remove(0);
                if self.connections.vacant_entry().is_none() {
                    let conns_len = self.connections.len();
                    self.connections.reserve_exact(conns_len);
                }

                let entry = self.connections.vacant_entry().unwrap();
                // if we are unable to register connection to this poll service
                // then just moving to the next connection, by just closing this one
                if !conn.register(&self.poll) {
                    drop(conn);
                    return;
                }

                entry.insert(conn);
            }
        }
    }

    #[inline(always)]
    fn readable(&mut self, token: Token) {
        if !self.connections.contains(token) {
            return;
        }

        {
            let ref mut conn = self.connections[token];

            loop {
                let (done, data_opt) = conn.read_data();

                // if we not done with reading but we don't have data anymore
                // just returning and waiting for the next data loop
                if !done {
                    return;
                }

                // if we done with reading and data is None returned
                // then we got error for socket, so just closing connection
                if data_opt.is_none() {
                    break;
                }

                // parsing event from given data
                let event = match Event::from_raw(&data_opt.unwrap()) {
                    Ok(e) => e,
                    Err(e) => {
                        warn!("Unable to convert recived data to event! -> {}", e);
                        return;
                    }
                };

                // sending event to Node service to process over handlers
                // then send over networking if we need to send it
                let _ = self.node_channel.send(NodeCommand{
                    cmd: NodeCMD::HandleDataEvent,
                    event: vec![event],
                });
            }
        }

        // if we got here then we want to close connection
        self.close_connection(token);
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
