#![allow(dead_code)]
extern crate mio;
extern crate threadpool;

use self::mio::channel::{channel, Receiver, Sender};
use network::tcp::{Slab, TcpReaderConn, CONNECTION_COUNT_PRE_ALLOC};
use network::{NetworkCommand, NetworkCMD
              , RECEIVER_CHANNEL_TOKEN, LOOP_EVENTS_COUNT};
use self::mio::{Poll, Ready, PollOpt, Events, Token};
use std::process;
use helper::Log;
use std::error::Error;
use node::Event;
use self::threadpool::ThreadPool;

pub enum TcpReaderCMD {
    NONE,
    HandleConnection
}

pub struct TcpReaderCommand {
    pub cmd: TcpReaderCMD,
    pub conn: Vec<TcpReaderConn>
}

pub struct TcpReader {
    // channels for TcpReader
    sender_chan: Sender<TcpReaderCommand>,
    receiver_chan: Receiver<TcpReaderCommand>,

    // List of connections for working with this TcpReader
    connections: Slab<TcpReaderConn>,

    // channel for base networking/node for sending parsed data to it
    net_chan: Sender<NetworkCommand>,

    // poll service for current reader
    poll: Poll
}

impl TcpReaderCommand {
    pub fn default() -> TcpReaderCommand {
        TcpReaderCommand {
            cmd: TcpReaderCMD::NONE,
            conn: vec![]
        }
    }
}

impl TcpReader {
    pub fn new(net_chan: Sender<NetworkCommand>) -> TcpReader {
        let (s, r) = channel::<TcpReaderCommand>();
        TcpReader {
            net_chan: net_chan,
            sender_chan: s,
            receiver_chan: r,
            connections: Slab::with_capacity(CONNECTION_COUNT_PRE_ALLOC),
            poll: match Poll::new() {
                Ok(p) => p,
                Err(e) => {
                    Log::error("Unable to make a Poll service for TCP reader", e.description());
                    process::exit(1);
                }
            }
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpReaderCommand> {
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
                    if kind == Ready::readable() {
                        self.readable(token, &thread_pool);
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
    fn notify(&mut self, command: &mut TcpReaderCommand) {
        match command.cmd {
            TcpReaderCMD::HandleConnection => {
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
            TcpReaderCMD::NONE => {}
        }
    }

    #[inline(always)]
    fn readable(&mut self, token: Token, thread_pool: &ThreadPool) {
        let (close_conn, data_list, conn_token) = {
            let ref mut conn = self.connections[token];
            match conn.read_data() {
                Some(d) => (false, d, conn.conn_token.clone()),
                None => {
                    // if we got None then there is something wrong with this connection
                    // we need to close it
                    (true, vec![], String::new())
                }
            }
        };

        if close_conn {
            self.close_connection(token);
            return;
        }

        if data_list.len() == 0 {
            return;
        }

        let net_chan = self.net_chan.clone();

        // making data parse and command send using separate thread pool
        thread_pool.execute(move || {
            let mut net_cmd = NetworkCommand::default();
            net_cmd.cmd = NetworkCMD::HandleEvent;
            net_cmd.token = vec![conn_token];
            let mut events: Vec<Event> = vec![];
            for data in data_list {
                events.push(match Event::from_raw(&data) {
                    Some(e) => e,
                    None => continue
                });
            }

            match net_chan.send(net_cmd) {
                Ok(_) => {},
                Err(e) => Log::error("Unable to send data over networking channel from TCP Reader", e.description())
            }
        });
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
}