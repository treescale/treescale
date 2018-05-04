#![allow(dead_code)]
extern crate mio;

use std::process;
use std::error::Error;
use std::sync::Arc;

use network::tcp::TcpConnection;
use network::{NetworkCommand, NetworkCMD, Slab, CONNECTION_COUNT_PRE_ALLOC, ConnectionIdentity, SocketType, Connection};
use node::{NET_RECEIVER_CHANNEL_TOKEN, EVENT_LOOP_EVENTS_SIZE};
use event::Event;
use helper::{Log, NetHelper};

use self::mio::channel::{Sender, Receiver, channel};
use self::mio::{Poll, Ready, PollOpt, Token, Events};

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

    // keeping index for this handler for later identification
    index: usize,
}

impl TcpHandler {
    /// Making new TCP handler service
    pub fn new(net_chan: Sender<NetworkCommand>, index: usize) -> TcpHandler {

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
            index: index
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpHandlerCommand> {
        self.sender_chan.clone()
    }

    /// Main function to start TCP Handler service as a separate thread if needed
    pub fn start(&mut self) {
        match self.poll.register(&self.receiver_chan, NET_RECEIVER_CHANNEL_TOKEN, Ready::readable(), PollOpt::level()) {
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
                    // if we got error, then data is unavailable
                    // and breaking receive loop
                    while let Ok(cmd) = self.receiver_chan.try_recv() {
                        let mut c = cmd;
                        self.notify(&mut c);
                    }

                    continue;
                }

                // we tracking events only for our connections
                if self.connections.contains(token) {
                    // if we got some error on one of the connections
                    // we need to close them
                    if kind.is_error() || kind.is_hup() {
                        self.close_connection(token);
                        continue;
                    }

                    // we only looking for readable connections
                    if kind.is_readable() {
                        self.readable(token);
                        continue;
                    }

                    if kind.is_writable() {
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

                    // adding connection to our connections list
                    conn.socket_token = entry.index();

                    // registering and making connection writable first
                    // just to clear write queue from the beginning
                    if !conn.register(&self.poll) {
                        Log::warn("Unable register TCP connection with TcpHandler POLL service", "Got connection by TcpHandleCommand");
                        continue;
                    }

                    // if connection is from client, then first of all we need to write handshake information
                    if !conn.from_server {
                        if !conn.make_writable(&self.poll) {
                            Log::warn("Unable make writable TCP connection with TcpHandler POLL service", "Got connection by TcpHandleCommand");
                            continue;
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
        let accepted = {
            let ref conn: TcpConnection = self.connections[token];
            Connection::check_api_version(conn.api_version) && conn.conn_token.len() > 0
        };

        if !accepted {
            // if we don't have handshake information
            // trying to read again
            if !self.read_handshake_info(token) {
                return
            }

            // if we got handshake information and connection is from server
            // making writable to send our handshake information
            let ref conn: TcpConnection = self.connections[token];
            if conn.from_server {
                conn.make_writable(&self.poll);
            }

            self.accept_connection(token);
            return
        }

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

        let mut event_cmd = NetworkCommand::new();
        event_cmd.cmd = NetworkCMD::HandleEvent;
        event_cmd.token = vec![conn_token];
        event_cmd.event.reserve_exact(data_list.len());
        for data in data_list {
            event_cmd.event.push(match Event::from_raw(&data) {
                Some(e) => e,
                None => continue
            });
        }

        match self.net_chan.send(event_cmd) {
            Ok(_) => {},
            Err(e) => Log::error("Unable to send data over networking channel from TCP Reader", e.description())
        }
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
            return
        }
    }

    #[inline(always)]
    fn close_connection(&mut self, token: Token) {
        // sending command to Networking that connection closed
        // or at least one channel was closed for this connection
        {
            let ref conn = self.connections[token];
            // if we have accepted connection, notifying about close action
            if Connection::check_api_version(conn.api_version) && conn.conn_token.len() > 0 {
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
        }
        self.connections.remove(token);
    }

    #[inline(always)]
    fn read_handshake_info(&mut self, token: Token) -> bool {
        // if we got here then we have connection with this token
        let mut close_conn = {
            let ref mut conn: TcpConnection = self.connections[token];
            // if we don't have yet API version defined
            if !Connection::check_api_version(conn.api_version) {
                match conn.read_api_version() {
                    Some((done, version)) => {
                        // if we not done with reading API version
                        // Just returning and waiting until next readable cycle
                        if !done {
                            return false;
                        }

                        // if we got wrong API version just closing connection
                        if !Connection::check_api_version(version) {
                            true
                        } else {
                            // if we got valid API version
                            // saving it as a connection version
                            conn.api_version = version;
                            false
                        }
                    }

                    // if we have connection error closing it
                    None => true
                }
            } else {
                false
            }
        };

        if close_conn {
            self.close_connection(token);
            return false;
        }

        close_conn = {
            let ref mut conn: TcpConnection = self.connections[token];
            // if we don't have token and value form connection
            if conn.conn_token.len() == 0 {
                // reading Connection Token and Value
                match conn.read_token_value() {
                    Some((done, token_str, value)) => {
                        // if we not done with reading API version
                        // Just returning and waiting until next readable cycle
                        if !done {
                            return false;
                        }

                        // checking if we got valid Prime Value or not
                        // if it's invalid just closing connection
                        if !NetHelper::validate_value(value) {
                            true
                        } else {
                            // if we done with token and value
                            // just setting them for connection
                            // and writing API handshake information
                            conn.conn_token = token_str;
                            conn.conn_value = value;

                            false
                        }
                    }

                    // if we have connection error closing it
                    None => true
                }
            } else {
                false
            }
        };

        if close_conn {
            self.close_connection(token);
            return false;
        }

        true
    }

    #[inline(always)]
    fn accept_connection(&self, token: Token) {
        let ref conn = self.connections[token];
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
    }
}