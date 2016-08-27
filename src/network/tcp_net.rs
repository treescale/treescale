extern crate mio;

use mio::{Handler, EventLoop, Token, EventSet, PollOpt};
use mio::tcp::{TcpStream};
use network::tcp_server::TcpServer;
use network::tcp_conn::TcpConnection;
use mio::util::Slab;
use error::error::Error;
use error::codes::ErrorCodes;
use std::io;

const SERVER_TOKEN: Token = Token(1);

const INVALID_TOKEN: Token = Token(0);

struct TcpNetwork {
    server: TcpServer,
    connections: Slab<TcpConnection>,
}

impl Handler for TcpNetwork {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<TcpNetwork>, token: Token, events: EventSet) {
        // If we got here invalid token handling error and just returning
        if token == INVALID_TOKEN {
            Error::handle_error(ErrorCodes::InvalidNetworkToken, "Invalid Token received from EventLoop", "Networking Ready state");
            return;
        }

        if events.is_error() {
            Error::handle_error(ErrorCodes::NetworkErrorEvent, "Error event from Networking Evenet Loop", "Networking Ready state");
            self.reset_connection(event_loop, token);
            return;
        }

        if events.is_readable() {
            if token == SERVER_TOKEN {
                self.accept_connection(event_loop);
            } else {
                // Read from connection
            }
        }

    }
}

impl TcpNetwork {
    fn reset_connection(&mut self, event_loop: &mut EventLoop<TcpNetwork>, token: Token) {
        if SERVER_TOKEN == token {
            event_loop.shutdown();
        } else {
            // TODO: remove connection with token if we got here
            // self.conns.remove(token);
        }
    }

    /// Find a connection in the slab using the given token.
    fn find_connection_by_token<'a>(&'a mut self, token: Token) -> &'a mut TcpConnection {
        &mut self.connections[token]
    }

    fn register_server(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()>  {
        event_loop.register(
            &self.server.sock,
            SERVER_TOKEN,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            Error::handle_error(ErrorCodes::NetworkTcpServerRegister, "Error registering server inside Networking EventLoop", "Networking Register");
            Err(e)
        })
    }

    fn reregister_server(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()>  {
        event_loop.reregister(
            &self.server.sock,
            SERVER_TOKEN,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            Error::handle_error(ErrorCodes::NetworkTcpServerRegister, "Error ReRegistering server inside Networking EventLoop", "Networking ReRegister");
            Err(e)
        })
    }

    fn accept_connection(&mut self, event_loop: &mut EventLoop<TcpNetwork>) {
        let sock = match self.server.sock.accept() {
            Ok(s) => {
                match s {
                    Some((sock, _)) => sock,
                    None => {
                        Error::handle_error(ErrorCodes::NetworkTcpConnectionAccept, "Unable to accept socket connection from server", "Networking Accept connection");
                        self.reregister_server(event_loop);
                        return;
                    }
                }
            }
            Err(e) => {
                Error::handle_error(ErrorCodes::NetworkTcpConnectionAccept, "Unable to accept socket connection from server", "Networking Accept connection");
                self.reregister_server(event_loop);
                return;
            }
        };

        match self.connections.insert_with(|token| {
            TcpConnection::new(sock, token, true)
        }) {
            Some(token) => {
                // if we got here then we successfully inserted connection
                // now we need to register it
                // match self.find_connection_by_token(token).register() {
                //
                // }
            },
            None => {
                Error::handle_error(ErrorCodes::NetworkTcpConnectionAccept, "Error inerting connection", "Networking Accept connection");
            }
        }
    }
}
