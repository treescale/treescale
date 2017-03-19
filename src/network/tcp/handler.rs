#![allow(dead_code)]
extern crate mio;
extern crate threadpool;

use std::process;
use std::error::Error;
use std::sync::Arc;

use network::tcp::TcpConnection;
use network::{NetworkCommand, NetworkCMD, Slab, CONNECTION_COUNT_PRE_ALLOC, ConnectionIdentity, SocketType};
use node::{NET_RECEIVER_CHANNEL_TOKEN, EVENT_LOOP_EVENTS_SIZE};
use event::Event;
use helper::Log;

use self::mio::channel::{Sender, Receiver, channel};
use self::mio::{Poll, Ready, PollOpt, Token, Events};
use self::threadpool::ThreadPool;

pub enum TcpHandlerCMD {
    None,
    HandleConnection,
    WriteData
}

pub struct TcpHandlerCommand {
    pub cmd: TcpHandlerCMD,
    pub conn: Vec<TcpConnection>,
    pub token: Vec<Token>,
    pub data: Vec<Arc<Vec<u8>>>

}

impl TcpHandlerCommand {
    pub fn new() -> TcpHandlerCommand {
        TcpHandlerCommand {
            cmd: TcpHandlerCMD::None,
            conn: vec![],
            data: vec![],
            token: vec![]
        }
    }
}

/// Main struct for handling TCP connections separately for reading and writing
pub struct TcpHandler {
    // Connections for current handler
    connections: Slab<TcpConnection>,

    // channels for TcpHandler
    sender_chan: Sender<TcpHandlerCommand>,
    receiver_chan: Receiver<TcpHandlerCommand>,

    // channel for networking
    net_chan: Sender<NetworkCommand>,

    // poll service for current writer
    poll: Poll,

    // keeping thread pool created in Node service
    thread_pool: ThreadPool,

    // keeping index for this handler for later identification
    index: usize,
}

impl TcpHandler {
    /// Making new TCP handler service
    pub fn new(net_chan: Sender<NetworkCommand>
               , thread_pool: ThreadPool, index: usize) -> TcpHandler {

        let (s, r) = channel::<TcpHandlerCommand>();

        TcpHandler {
            connections: Slab::with_capacity(CONNECTION_COUNT_PRE_ALLOC),
            sender_chan: s,
            receiver_chan: r,
            net_chan: net_chan,
            poll: match Poll::new() {
                Ok(p) => p,
                Err(e) => {
                    Log::error("Unable to make TcpHandler POLL service", e.description());
                    process::exit(1);
                }
            },
            thread_pool: thread_pool,
            index: index
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpHandlerCommand> {
        self.sender_chan.clone()
    }

    /// Main function to start TCP Handler service as a separate thread if needed
    pub fn start(&mut self) {
        match self.poll.register(&self.receiver_chan, NET_RECEIVER_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                Log::error("Unable to register TcpHandler receiver channel", e.description());
                process::exit(1);
            }
        }

        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(EVENT_LOOP_EVENTS_SIZE);
        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.iter() {
                let (token, kind) = (event.token(), event.kind());
                if token == NET_RECEIVER_CHANNEL_TOKEN {
                    // trying to get commands while there is available data
                    loop {
                        match self.receiver_chan.try_recv() {
                            Ok(cmd) => {
                                let mut c = cmd;
                                self.notify(&mut c);
                            }
                            // if we got error, then data is unavailable
                            // and breaking receive loop
                            Err(_) => {
                                break;
                            }
                        }
                    }

                    continue;
                }

                // we tracking events only for our connections
                if self.connections.contains(token) {
                    // if we got some error on one of the connections
                    // we need to close them
                    if kind.is_error() || kind.is_hup() {
                        println!("Event EE -> {:?}", token);
                        self.close_connection(token);
                        continue;
                    }

                    // we only looking for readable connections
                    if kind.is_readable() {
                        println!("Event R -> {:?}", token);
                        self.readable(token);
                        continue;
                    }

                    if kind.is_writable() {
                        println!("Event W -> {:?}", token);
                        self.writable(token);
                        continue;
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut TcpHandlerCommand) {
        match command.cmd {
            TcpHandlerCMD::HandleConnection => {
                while !command.conn.is_empty() {
                    // getting connection
                    let mut conn = command.conn.remove(0);

                    // if we don't have a space in our connections array, just allocating more space
                    if self.connections.vacant_entry().is_none() {
                        self.connections.reserve_exact(CONNECTION_COUNT_PRE_ALLOC);
                    }

                    let entry = match self.connections.vacant_entry() {
                        Some(e) => e,
                        None => {
                            Log::warn("Unable to allocate space for inserting TCP connection to TcpHandler", "Got connection by TcpHandleCommand");
                            continue;
                        }
                    };

                    // registering and making connection writable first
                    // just to clear write queue from the beginning
//                    if !conn.make_readable(&self.poll) {
//                        Log::warn("Unable register TCP connection with TcpHandler POLL service", "Got connection by TcpHandleCommand");
//                        continue;
//                    }

                    if !conn.make_writable(&self.poll) {
                        Log::warn("Unable make writable TCP connection with TcpHandler POLL service", "Got connection by TcpHandleCommand");
                        continue;
                    }

                    // adding connection to our connections list
                    conn.socket_token = entry.index();

                    // notifying Networking about new connection accepted
                    let mut net_cmd = NetworkCommand::new();
                    net_cmd.cmd = NetworkCMD::HandleConnection;
                    net_cmd.token.push(conn.conn_token.clone());
                    net_cmd.value.push(conn.conn_value);
                    net_cmd.conn_identity.push(ConnectionIdentity {
                        handler_index: self.index,
                        socket_type: SocketType::TCP,
                        socket_token: conn.socket_token
                    });
                    match self.net_chan.send(net_cmd) {
                        Ok(_) => {}
                        Err(e) => {
                            Log::error("Unable to send command to networking from TcpHandler"
                                       , format!("New TCP connection Command for Token - {} -> {}", conn.conn_token.clone(), e).as_str());
                        }
                    }

                    entry.insert(conn);
                }
            }

            TcpHandlerCMD::WriteData => {
                // picking up all connection that we are requested for
                while !command.token.is_empty() {
                    let token = command.token.remove(0);
                    if !self.connections.contains(token) {
                        continue;
                    }

                    let ref mut conn = self.connections[token];

                    // writing data to connection
                    // this will automatically make connection writable for poll service
                    for data in &command.data {
                        conn.write(data.clone(), &self.poll);
                    }
                }
            }
            TcpHandlerCMD::None => {}
        }
    }

    #[inline(always)]
    fn readable(&mut self, token: Token) {
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

        let channel = self.net_chan.clone();

        // making data parse and command send using separate thread pool
        self.thread_pool.execute(move || {
            let mut event_cmd = NetworkCommand::new();
            event_cmd.cmd = NetworkCMD::HandleEvent;
            event_cmd.token.push(conn_token);
            let mut events: Vec<Event> = vec![];
            for data in data_list {
                events.push(match Event::from_raw(&data) {
                    Some(e) => e,
                    None => continue
                });
            }

            match channel.send(event_cmd) {
                Ok(_) => {},
                Err(e) => Log::error("Unable to send data over networking channel from TCP Reader", e.description())
            }
        });
    }

    #[inline(always)]
    fn writable(&mut self, token: Token) {
        let close_conn = {
            let ref mut conn = self.connections[token];
            match conn.flush() {
                Some(done) => {
                    if done {
                        // if we are done with flushing write queue
                        // making connection readable again
                        conn.make_readable(&self.poll);
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
        // sending command to Networking that connection closed
        // or at least one channel was closed for this connection
        {
            let ref conn = self.connections[token];
            let mut net_cmd = NetworkCommand::new();
            net_cmd.cmd = NetworkCMD::ConnectionClose;
            net_cmd.token = vec![conn.conn_token.clone()];
            net_cmd.conn_identity.push(ConnectionIdentity {
                socket_type: SocketType::TCP,
                handler_index: self.index,
                socket_token: token
            });
            match self.net_chan.send(net_cmd) {
                Ok(_) => {}
                Err(e) => {
                    Log::error("Unable to send command to networking from TcpHandler"
                               , format!("Connection Close Command for Token - {} -> {}", conn.conn_token.clone(), e).as_str());
                }
            }
        }
        self.connections.remove(token);
    }
}