extern crate mio;

use mio::{Handler, EventLoop, Token, EventSet};
use mio::tcp::{TcpListener};
use network::tcp_server::{TcpServer, SERVER_TOKEN};
use network::tcp_conn::{TcpConns};
use network::tcp_reader::TcpReader;
use error::error::Error;
use error::codes::ErrorCodes;
use std::sync::Arc;

pub const INVALID_TOKEN: Token = Token(0);
const MAX_CONNECTIONS: usize = 100000000;

pub enum LoopCommand {
    STOP_LOOP,
    REMOVE_CONNECTION,
    ACCEPT_CONNECTION
}

pub struct NetLoopCmd {
    pub cmd: LoopCommand,
    pub token: Token
}

pub struct TcpNetwork {
    pub connections: TcpConns,
    pub is_api: bool,
    pub event_loop: EventLoop<TcpNetwork>,

    // main server socket
    pub server_sock: Vec<TcpListener>,
    pub server_address: String,

    // keeping TcpReaders for transfering connection to read process
    readers: Vec<TcpReader>,
    readers_index: usize
}


impl Handler for TcpNetwork {
    type Timeout = ();
    type Message = NetLoopCmd;

    fn ready(&mut self, event_loop: &mut EventLoop<TcpNetwork>, token: Token, events: EventSet) {
        // If we got here invalid token handling error and just returning
        if token == INVALID_TOKEN {
            Error::handle_error(ErrorCodes::InvalidNetworkToken, "Invalid Token received from EventLoop", "Networking Ready state");
            return;
        }

        if events.is_error() {
            Error::handle_error(ErrorCodes::NetworkErrorEvent, "Error event from Networking Event Loop", "Networking Ready state");
            self.reset_connection(event_loop, token);
            return;
        }

        if events.is_readable() {
            if token == SERVER_TOKEN {
                self.accept_connection(event_loop);
            } else {
                // finding connection here, reading some data and then registering to new events
                // if we got error during read process just reseting connection
                self.connections.find_connection_by_token(token)
                .and_then(|conn| conn.read_data_net(event_loop))
                .unwrap_or_else(|_| {
                    self.reset_connection(event_loop, token);
                })
            }
        }

        if events.is_writable() {
            // checking if we got write event for server or not
            // if it's true then just returning, because server can't have write event
            if token == SERVER_TOKEN {return;}

            // Writing data to available socket by token
            self.connections.find_connection_by_token(token)
            .and_then(|conn| conn.write_data_net(event_loop))
            .unwrap_or_else(|_| {
                self.reset_connection(event_loop, token);
            })
        }
    }

    // Handling commands here
    fn notify(&mut self, event_loop: &mut EventLoop<TcpNetwork>, cmd: NetLoopCmd) {
        // checking command type
        match cmd.cmd {
            LoopCommand::STOP_LOOP => event_loop.shutdown(),
            LoopCommand::REMOVE_CONNECTION => self.reset_connection(event_loop, cmd.token),
            LoopCommand::ACCEPT_CONNECTION => {
                // Writing data to available socket by token
                self.connections.find_connection_by_token(cmd.token)
                .and_then(|conn| {
                    event_loop.deregister(&conn.sock)
                    // Picup some reader by load balancing them
                })
                .unwrap_or_else(|_| {
                    // we don't care for this
                });
            }
        }
    }
}

impl TcpNetwork{

    pub fn new(server_address: &str, is_api: bool, readers_count: usize) -> Arc<TcpNetwork> {
        let net = Arc::new(TcpNetwork {
            connections: TcpConns::new(MAX_CONNECTIONS),
            server_sock: Vec::new(),
            is_api: is_api,
            server_address: String::from(server_address),
            readers_index: 0,
            readers: Vec::new(),
            event_loop: EventLoop::new().ok().expect("Unable to create event loop for networking")
        });
        let mut nn = net.clone();
        let mut n = Arc::get_mut(&mut nn).unwrap();

        // We need to add current network to networks list
        for i in 0..readers_count {
            n.readers.push(TcpReader::new(n.event_loop.channel(), net.clone()));
        };

        net
    }



    /// This function will start event loop and will register server if it's exists
    pub fn run(&mut self) {

    }

    /// Reset connection if we got some error from event loop
    /// this function is called from event loop side
    /// if token is server token, so we are shuting down event loop, so it will close all connections
    /// if token is for single connection just removing it from our list
    fn reset_connection(&mut self, event_loop: &mut EventLoop<TcpNetwork>, token: Token) {
        if SERVER_TOKEN == token {
            event_loop.shutdown();
        } else {
            self.connections.remove_connection_by_token(token);
        }
    }
}
