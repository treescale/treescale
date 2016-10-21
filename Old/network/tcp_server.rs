extern crate mio;

use mio::{EventLoop, PollOpt, EventSet, Token};
use error::error::Error;
use error::codes::ErrorCodes;
use network::tcp_net::TcpNetwork;
use std::io;

pub const SERVER_TOKEN: Token = Token(1);

pub trait TcpServer {
    /// Register TcpServer event for accepting connection
    /// this will be called from event loop ready function or from the beginning
    fn register_server(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()>;

    /// Reregister server socket for accepting connections and getting events
    fn reregister_server(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()>;

    /// Accept connection here, when event loop would be ready to accept
    /// this function is called from event loop
    fn accept_connection(&mut self, event_loop: &mut EventLoop<TcpNetwork>);
}

impl TcpServer for TcpNetwork {

    fn register_server(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()>  {
        // if we are in server and have already binded socket
        if self.is_api {
            Ok(())
        }
        else {
            event_loop.register(
                &self.server_sock,
                SERVER_TOKEN,
                EventSet::readable(),
                PollOpt::edge() | PollOpt::oneshot()
            ).or_else(|e| {
                Error::handle_error(ErrorCodes::NetworkTcpServerRegister, "Error registering server inside Networking EventLoop", "Networking Register");
                Err(e)
            })
        }
    }

    fn reregister_server(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()>  {
        // if we are in server and have already binded socket
        if self.is_api {
            Ok(())
        }
        else {
            event_loop.reregister(
                &self.server_sock,
                SERVER_TOKEN,
                EventSet::readable(),
                PollOpt::edge() | PollOpt::oneshot()
            ).or_else(|e| {
                Error::handle_error(ErrorCodes::NetworkTcpServerRegister, "Error ReRegistering server inside Networking EventLoop", "Networking ReRegister");
                Err(e)
            })
        }
    }


    fn accept_connection(&mut self, event_loop: &mut EventLoop<TcpNetwork>) {
        if self.is_api {
            return;
        }

        let sock = match self.server_sock.accept() {
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

        let res = {
            self.connections.insert_with(sock)
        };

        match res {
            Some(token) => {
                //if we got here then we successfully inserted connection
                //now we need to register it
                let st = match self.connections.find_connection_by_token(token) {
                    Ok(conn) => {
                        conn.register_net(event_loop)
                    },
                    Err(e) => Err(e)
                };

                match st {
                    Ok(_) => {},
                    Err(_) => {
                        // if we got error during reregister process just removing connection from list
                        self.connections.remove_connection_by_token(token);
                    }
                }
            },
            None => {
                Error::handle_error(ErrorCodes::NetworkTcpConnectionAccept, "Error inserting connection", "Networking Accept connection");
            }
        }
    }
}
