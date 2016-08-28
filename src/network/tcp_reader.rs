extern crate mio;

use mio::{EventLoop, Handler, Token, EventSet, Sender};
use network::tcp_net::{NetLoopCmd, INVALID_TOKEN, TcpNetwork, LoopCommand};
use network::tcp_server::SERVER_TOKEN;
use network::tcp_conn::{TcpConns};
use error::error::Error;
use error::codes::ErrorCodes;
use std::sync::Arc;
use std::io;

pub struct TcpReader {
    pub event_loop: EventLoop<TcpReader>,

    pub net_chanel: Sender<NetLoopCmd>,
    pub net: Arc<TcpNetwork>
}

/// TcpReader event loop will be only connection reader/writer loop
/// we should't have connection accept functionality
impl Handler for TcpReader {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<TcpReader>, token: Token, events: EventSet) {
        // If we got here invalid token handling error and just returning
        if token == INVALID_TOKEN {
            Error::handle_error(ErrorCodes::InvalidNetworkToken, "Invalid Token received from reader EventLoop", "TcpReader Ready state");
            return;
        }

        if token == SERVER_TOKEN {
            Error::handle_error(ErrorCodes::InvalidNetworkToken, "Server token recieved in reader EventLoop", "TcpReader Ready state");
            return;
        }

        if events.is_error() {
            Error::handle_error(ErrorCodes::NetworkErrorEvent, "Error event from Tcp reader Evenet Loop", "TcpReader Ready state");
            // If we got some error letting know about that to Networking loop to reset connection
            self.net_chanel.send(NetLoopCmd {
                cmd: LoopCommand::REMOVE_CONNECTION,
                token: token
            });
            return;
        }

        // extracting network pointer
        let mut nn = self.net.clone();
        let mut net = Arc::get_mut(&mut nn).unwrap();

        if events.is_readable() {
            if token == SERVER_TOKEN {
                // We shouldn't get Server token here
                return;
            } else {
                // finding connection here, reading some data and then registering to new events
                // if we got error during read process just reseting connection
                net.connections.find_connection_by_token(token)
                .and_then(|conn| conn.read_data_reader(event_loop))
                .unwrap_or_else(|_| {
                    self.net_chanel.send(NetLoopCmd {
                        cmd: LoopCommand::REMOVE_CONNECTION,
                        token: token
                    });
                })
            }
        }

        if events.is_writable() {
            // checking if we got write event for server or not
            // if it's true then just returning, because server can't have write event
            if token == SERVER_TOKEN {return;}

            // Writing data to available socket by token
            net.connections.find_connection_by_token(token)
            .and_then(|conn| conn.write_data_reader(event_loop))
            .unwrap_or_else(|_| {
                self.net_chanel.send(NetLoopCmd {
                    cmd: LoopCommand::REMOVE_CONNECTION,
                    token: token
                });
            })
        }
    }
}

impl TcpReader {
    pub fn new(net_chan: Sender<NetLoopCmd>, net: Arc<TcpNetwork>) -> TcpReader {
        TcpReader {
            event_loop: EventLoop::new().ok().expect("Unable to create event loop for reader"),
            net_chanel: net_chan,
            net: net
        }
    }

    /// Transfer connection to reader loop
    /// it is required to call event_loop.deregister for this connection before calling this function, for thread safety
    pub fn transfer_connection(&mut self, token: Token) -> io::Result<()> {
        // extracting network pointer
        let mut nn = self.net.clone();
        let mut net = Arc::get_mut(&mut nn).unwrap();

        net.connections.find_connection_by_token(token)
        .and_then(|conn| conn.register_reader(&mut self.event_loop))
        .unwrap_or_else(|_| {
            Error::handle_error(ErrorCodes::NetworkErrorEvent, "Unable to transfer connection to Reader loop", "Reader Transfer connection");
            self.net_chanel.send(NetLoopCmd {
                cmd: LoopCommand::REMOVE_CONNECTION,
                token: token
            });
        });

        Ok(())
    }
}
