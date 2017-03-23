#![allow(dead_code)]
extern crate mio;

use helper::Log;

use self::mio::tcp::{TcpListener, TcpStream};
use self::mio::{Ready, PollOpt, Token};
use self::mio::channel::Sender;

use node::{Node, NET_TCP_SERVER_TOKEN};
use network::{TcpConnection
              , TcpHandler, Networking
              , TcpHandlerCommand, TcpHandlerCMD};


use std::net::SocketAddr;
use std::error::Error;
use std::process;
use std::str::FromStr;
use std::io::ErrorKind;
use std::thread;
use std::sync::Arc;

/// TcpNetwork Trait for implementing TCP networking capabilities
/// On top of Node structure
pub trait TcpNetwork {
    /// Min function to attach TCP service functionality to existing POLL service
    fn register_tcp(&mut self);

    /// Make TCP server socket listener from given address
    fn make_tcp_server(address: &str) -> TcpListener;

    /// Handler for event loop ready event
    /// This is general event processing for TCP connections/servers
    /// If event token not in the TCP list it will return false
    /// To let other components to handle event
    fn tcp_ready(&mut self, token: Token, event_kind: Ready) -> bool;

    /// Function for accepting TCP connections as a pending connections
    fn tcp_acceptable(&mut self);

    /// getting one of the TCP handler channels
    /// using Round Rubin algorithm
    fn tcp_get_handler(&mut self) -> Sender<TcpHandlerCommand>;

    /// making client connection to given address
    fn tcp_connect(&mut self, address: &str) -> bool;

    /// Transferring connection from pending to one of the TCP handlers
    fn tcp_transfer_connection(&mut self, sock: TcpStream, from_server: bool);
}

impl TcpNetwork for Node {
    fn register_tcp(&mut self) {
        match self.poll.register(&self.net_tcp_server, NET_TCP_SERVER_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {}
            Err(e) => {
                Log::error("Unable to register TCP server to Node POLL service", e.description());
                process::exit(1);
            }
        }

        // making TCP handlers based on initial allocated capacity
        let handlers_count = self.net_tcp_handler_sender_chan.capacity();
        if handlers_count == 0 {
            Log::warn("There is no concurrency defined, exiting process!", "From TcpNetworking Registering functionality");
            process::exit(1);
        }

        for i in 0..handlers_count {
            let mut handler = TcpHandler::new(self.net_sender_chan.clone(), i);
            self.net_tcp_handler_sender_chan.push(handler.channel());
            thread::spawn(move || {
                handler.start();
            });
        }
    }

    fn make_tcp_server(address: &str) -> TcpListener {
        let addr = match SocketAddr::from_str(address) {
            Ok(a) => a,
            Err(e) => {
                Log::error("Unable to parse given TCP server address", e.description());
                process::exit(1);
            }
        };

        match TcpListener::bind(&addr) {
            Ok(s) => s,
            Err(e) => {
                Log::error("Unable to bind given TCP server address", e.description());
                process::exit(1);
            }
        }
    }

    #[inline(always)]
    fn tcp_ready(&mut self, token: Token, event_kind: Ready) -> bool {
        if token == NET_TCP_SERVER_TOKEN {
            if event_kind != Ready::readable() {
                Log::error("Unexpected TCP Server event kind", "Ignoring for now!");
                return false;
            }

            self.tcp_acceptable();
            return true;
        }

        false
    }

    #[inline(always)]
    fn tcp_acceptable(&mut self) {
        loop {
            let sock = match self.net_tcp_server.accept() {
                Ok((s, _)) => s,
                Err(e) => {
                    // if we got WouldBlock, then this is Non Blocking socket
                    // and data still not available for this, so it's not a connection error
                    if e.kind() != ErrorKind::WouldBlock {
                        Log::error("Unable to accept connection from TCP server socket", e.description());
                    }
                    return;
                }
            };

            self.tcp_transfer_connection(sock, true);
        };
    }

    #[inline(always)]
    fn tcp_get_handler(&mut self) -> Sender<TcpHandlerCommand> {
        if self.net_tcp_handler_index >= self.net_tcp_handler_sender_chan.len() {
            self.net_tcp_handler_index = 0;
        }

        let i = self.net_tcp_handler_index;
        self.net_tcp_handler_index += 1;

        self.net_tcp_handler_sender_chan[i].clone()
    }

    #[inline(always)]
    fn tcp_connect(&mut self, address: &str) -> bool {
        let sock_address = match SocketAddr::from_str(address) {
            Ok(a) => a,
            Err(e) => {
                Log::error(format!("Unable to parse address for making connection to TCP server {}", address).as_str(), e.description());
                return false;
            }
        };

        let sock = match TcpStream::connect(&sock_address) {
            Ok(s) => s,
            Err(e) => {
                Log::error(format!("Unable to connect with given tcp address {}", address).as_str(), e.description());
                return false;
            }
        };

        self.tcp_transfer_connection(sock, false);
        true
    }

    #[inline(always)]
    fn tcp_transfer_connection(&mut self, sock: TcpStream, from_server: bool) {
        let mut command = TcpHandlerCommand::new();
        command.cmd = TcpHandlerCMD::HandleConnection;
        command.conn.push(TcpConnection::new(sock, Token(0), from_server));
        // adding handshake info, for writing it later from handler
        command.conn[0].add_writable_data(Arc::new(self.handshake_info()));
        match self.tcp_get_handler().send(command) {
            Ok(_) => {},
            Err(e) => {
                Log::error("Unable to send HandleConnection command to TCP handler", e.description());
                return;
            }
        }
    }
}

