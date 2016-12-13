#![allow(dead_code)]
extern crate mio;
extern crate num;

use std::sync::{Arc, RwLock};
use std::collections::BTreeMap;
use self::mio::{Token, Poll, Ready, PollOpt, Events};
use self::mio::channel::{channel, Sender, Receiver};
use network::tcp::{TcpConnection};
use event::{Event};
use self::num::{BigInt, Zero};
use std::str::FromStr;

const RECEIVER_CHANNEL_TOKEN: Token = Token(1);

pub enum TcpReaderCMD {
    HandleConnection,
    WriteWithPath,
    WriteWithToken,
}

pub struct TcpReaderCommand {
    pub cmd: TcpReaderCMD,
    pub conn: Vec<TcpConnection>,
    pub token: Vec<String>,
    pub event: Vec<Event>
}

pub struct TcpReader {
    // base list of connections,
    // which should be comming from TcpNetworking
    // TcpReader would use this only in read only mode
    connections: Arc<RwLock<BTreeMap<Token, TcpConnection>>>,
    connections_write_queue: BTreeMap<Token, Vec<Arc<Vec<u8>>>>,

    // channels for thread communication
    sender_channel: Sender<TcpReaderCommand>,
    receiver_channel: Receiver<TcpReaderCommand>,

    poll: Poll
}

impl TcpReader {
    pub fn new(connections: Arc<RwLock<BTreeMap<Token, TcpConnection>>>) -> TcpReader {
        let (s, r) = channel::<TcpReaderCommand>();
        TcpReader {
            connections: connections,
            sender_channel: s,
            receiver_channel: r,
            poll: Poll::new().unwrap(),
            connections_write_queue: BTreeMap::new()
        }
    }

    pub fn channel(&self) -> Sender<TcpReaderCommand> {
        self.sender_channel.clone()
    }

    pub fn run(&mut self) {
        match self.poll.register(&self.receiver_channel, RECEIVER_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                warn!("Unable to register receiver channel to Poll service for Reader -> {}", e);
                return;
            }
        }

        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(5000);
        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.into_iter() {
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

                if kind == Ready::error() || kind == Ready::hup() {
                    // if this error on connection, then we need to close it
                    self.close_connection(token);
                    continue;
                }

                if kind == Ready::readable() {
                    self.readable(token);
                    continue;
                }

                if kind == Ready::writable() {
                    self.writable(token);
                    continue;
                }
            }
        }
    }

    #[inline(always)]
    fn close_connection(&mut self, token: Token) {
        let mut locked_conns = match self.connections.write() {
            Ok(l) => l,
            Err(e) => {
                warn!("Unable to lock connections for closing connection from Reader -> {}", e);
                return;
            }
        };

        match locked_conns.remove(&token) {
            Some(_) => {},
            None => return
        }

        // if we got here then we removed connection
        // now we need trigger some event about it

        // TODO: trigger event about connection close !!
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut TcpReaderCommand) {
        match command.cmd {
            TcpReaderCMD::HandleConnection => {
                loop {
                    let conn = match command.conn.pop() {
                        Some(c) => c,
                        None => return
                    };

                    // if we got here then we already have connection
                    // inserting to connections list and registering to event loop
                    // NOTE: during first time registeration making connection writable just for flushing remaining data
                    match self.poll.register(&conn.socket, conn.socket_token, Ready::readable() | Ready::writable(), PollOpt::edge()) {
                        Ok(_) => {},
                        Err(e) => {
                            warn!("Unable to register connection to Reader event loop -> {}", e);
                            // after this connection would be automatically deleted
                            // and connections would be closed if it is exists
                            continue;
                        }
                    }

                    // after this scope our write lock would be deallocated and lock would be freed
                    {
                        let mut locked_conns = match self.connections.write() {
                            Ok(c) => c,
                            Err(e) => {
                                warn!("Unable to lock connections for adding new connection to it -> {}", e);
                                // after this connection would be automatically deleted
                                // and connections would be closed if it is exists
                                continue;
                            }
                        };

                        // inserting connection
                        locked_conns.insert(conn.socket_token, conn);
                    }
                }
            }

            TcpReaderCMD::WriteWithPath => {
                loop {
                    let mut ev = match command.event.pop() {
                        Some(p) => p,
                        None => return
                    };

                    let mut path = match BigInt::from_str(ev.path.as_str()) {
                        Ok(p) => p,
                        Err(_) => return
                    };

                    let mut conn_tokens: Vec<Token> = vec![];

                    // locking connections for readable lock
                    {
                        let locked_conns = match self.connections.read() {
                            Ok(c) => c,
                            Err(e) => {
                                warn!("Unable to lock connections for reading 1 -> {}", e);
                                return;
                            }
                        };

                        for (t, conn) in locked_conns.iter() {
                            // ignoring API connections
                            if conn.value == Zero::zero() {
                                continue;
                            }

                            if path.clone() % conn.value.clone() == Zero::zero() {
                                path = path.clone() / conn.value.clone();
                                conn_tokens.push(*t);
                            }
                        }
                    }

                    ev.path = path.to_str_radix(10);

                    let send_data = Arc::new(match ev.to_raw() {
                        Ok(d) => d,
                        Err(e) => {
                            warn!("Unable to convert event to Raw data from Reader Emmit -> {}", e);
                            return;
                        }
                    });

                    // locking connections for reading socket information from them
                    {
                        let locked_conns = match self.connections.read() {
                            Ok(c) => c,
                            Err(e) => {
                                warn!("Unable to lock connections for reading 2 -> {}", e);
                                return;
                            }
                        };

                        for token in conn_tokens.iter() {
                            match locked_conns.get(token) {
                                Some(c) => {
                                    // registering connection as writable
                                    match self.poll.reregister(&c.socket, c.socket_token, Ready::readable() | Ready::writable(), PollOpt::edge()) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            warn!("Unable to reregister connection as writable Reader Emmit -> {}", e);
                                            continue
                                        }
                                    };

                                    let mut q = match self.connections_write_queue.remove(token) {
                                        Some(q) => q,
                                        None => Vec::new()
                                    };

                                    q.push(send_data.clone());
                                    self.connections_write_queue.insert(*token, q);
                                }

                                None => continue
                            };
                        }
                    }
                }
            }

            TcpReaderCMD::WriteWithToken => {

            }
        }
    }

    #[inline(always)]
    fn readable(&mut self, token: Token) {

    }

    #[inline(always)]
    fn writable(&mut self, token: Token) {

    }
}
