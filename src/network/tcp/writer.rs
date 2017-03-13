#![allow(dead_code)]
extern crate mio;
extern crate threadpool;

use self::mio::channel::{channel, Receiver, Sender};
use network::tcp::{Slab, TcpWriterConn, CONNECTION_COUNT_PRE_ALLOC};
use network::{NetworkCommand, NetworkCMD
              , RECEIVER_CHANNEL_TOKEN, LOOP_EVENTS_COUNT};
use self::mio::{Poll, Ready, PollOpt, Events, Token};
use std::process;
use helper::Log;
use std::error::Error;
use self::threadpool::ThreadPool;
use std::sync::Arc;

pub enum TcpWriterCMD {
    NONE,
    HandleConnection,
    WriteData
}

pub struct TcpWriterCommand {
    pub cmd: TcpWriterCMD,
    pub conn: Vec<TcpWriterConn>,
    pub socket_token: Vec<Token>,
    pub data: Vec<Arc<Vec<u8>>>
}

pub struct TcpWriter {
    // channels for TcpReader
    sender_chan: Sender<TcpWriterCommand>,
    receiver_chan: Receiver<TcpWriterCommand>,

    // List of connections for working with this TcpReader
    connections: Slab<TcpWriterConn>,

    // channel for base networking/node for sending parsed data to it
    net_chan: Sender<NetworkCommand>,

    // poll service for current writer
    poll: Poll,
}

impl TcpWriterCommand {
    pub fn default() -> TcpWriterCommand {
        TcpWriterCommand {
            cmd: TcpWriterCMD::NONE,
            conn: vec![],
            socket_token: vec![],
            data: vec![]
        }
    }
}

impl TcpWriter {
    pub fn new(net_chan: Sender<NetworkCommand>) -> TcpWriter {
        let (s, r) = channel::<TcpWriterCommand>();
        TcpWriter {
            net_chan: net_chan,
            sender_chan: s,
            receiver_chan: r,
            connections: Slab::with_capacity(CONNECTION_COUNT_PRE_ALLOC),
            poll: match Poll::new() {
                Ok(p) => p,
                Err(e) => {
                    Log::error("Unable to make a Poll service for TCP writer", e.description());
                    process::exit(1);
                }
            }
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpWriterCommand> {
        self.sender_chan.clone()
    }

    pub fn start(&mut self, thread_pool: ThreadPool) {
        match self.poll.register(&self.receiver_chan, RECEIVER_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                Log::error("Unable to register TcpReader Receiver Channel", e.description());
                process::exit(1);
            }
        }

        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(LOOP_EVENTS_COUNT);
        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.iter() {
                let (token, kind) = (event.token(), event.kind());
                if token == RECEIVER_CHANNEL_TOKEN {
                    // trying to get commands while there is available data
                    loop {
                        match self.receiver_chan.try_recv() {
                            Ok(cmd) => {
                                let mut c = cmd;
                                self.notify(&mut c);
                            }
                            // if we got error, then data is unavailable
                            // and breaking receive loop
                            Err(e) => {
                                Log::warn("TcpReader receiver channel data is not available",
                                          e.description());
                                break;
                            }
                        }
                    }

                    continue;
                }

                // we tracking events only for our connections
                if self.connections.contains(token) {
                    // we only looking for readable connections
                    if kind == Ready::writable() {
                        self.writable(token, &thread_pool);
                        continue;
                    }

                    // if we got some error on one of the connections
                    // we need to close them
                    if kind == Ready::error() || kind == Ready::hup() {
                        self.close_connection(token);
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut TcpWriterCommand) {
        match command.cmd {
            TcpWriterCMD::HandleConnection => {
                if command.conn.len() == 0 {
                    return;
                }

                // we will transfer only one connection at a time
                let mut conn = command.conn.remove(0);

                // if we don't have a space in our connections array, just allocating more space
                if self.connections.vacant_entry().is_none() {
                    self.connections.reserve_exact(CONNECTION_COUNT_PRE_ALLOC);
                }

                let entry = self.connections.vacant_entry().unwrap();
                let token = entry.index();
                // setting new socket token
                conn.socket_token = token;
                entry.insert(conn);
            }
            TcpWriterCMD::WriteData => {
                if command.socket_token.len() == 0 {
                    return;
                }

                // we will send data only one connection at a time
                let token = command.socket_token.remove(0);
                if !self.connections.contains(token) {
                    return;
                }

                {
                    let ref mut conn = self.connections[token];
                    while !command.data.is_empty() {
                        // popping out first element
                        conn.write(command.data.remove(0));
                    }
                }

                // making writable to flush added content
                self.make_writable(&self.connections[token]);
            }
            TcpWriterCMD::NONE => {}
        }
    }

    #[inline(always)]
    fn writable(&mut self, token: Token, _: &ThreadPool) {
        let close_conn = {
            let ref mut conn = self.connections[token];
            match conn.flush() {
                Some(done) => {
                    if done {
                        // if we done, just deregistering connection for not getting writable events
                        let _ = self.poll.deregister(&conn.socket);
                    }

                    false
                },
                None => true
            }
        };

        if close_conn {
            self.close_connection(token);
        }
    }

    #[inline(always)]
    fn close_connection(&mut self, token: Token) {
        if self.connections.contains(token) {
            // informing networking that this connection is closed
            let mut net_cmd = NetworkCommand::default();
            net_cmd.token = vec![self.connections[token].conn_token.clone()];
            net_cmd.cmd = NetworkCMD::CloseConnection;
            let _ = self.net_chan.send(net_cmd);

            // shutting down socket
            self.connections[token].close();
            // then removing it and closing connection with it
            self.connections.remove(token);
        }
    }

    #[inline(always)]
    fn make_writable(&self, conn: &TcpWriterConn) {
        match self.poll.reregister(&conn.socket, conn.socket_token, Ready::writable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => Log::error("Unable to re-register connection as writable for TcpWriter POLL", e.description())
        }
    }
}