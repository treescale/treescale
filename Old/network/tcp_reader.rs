extern crate mio;
extern crate num;

use mio::{EventLoop, Handler, Token, EventSet, Sender};
use mio::tcp::TcpStream;
use network::tcp_net::{NetLoopCmd, INVALID_TOKEN, TcpNetwork, LoopCommand, MAX_CONNECTIONS};
use network::tcp_server::SERVER_TOKEN;
use network::tcp_conn::{TcpConns};
use error::error::Error;
use error::codes::ErrorCodes;
use std::sync::Arc;
use std::io;
use std::thread;
use self::num::bigint::{BigInt, Sign};
use self::num::Zero;
use self::num::integer::Integer;

pub enum ReaderCommands {
    STOP_LOOP,
    HANDLE_CONNECTION,
    WRITE_DATA
}

pub struct ReaderLoopCommand {
    pub cmd: ReaderCommands,
    pub conn_socks: Vec<TcpStream>,
    pub write_data: Vec<Arc<Vec<u8>>>,
    pub write_path: Vec<BigInt>
}

pub struct TcpReader {
    pub net_chanel: Sender<NetLoopCmd>,
    pub conns: TcpConns,
    pub loop_channel: Vec<Sender<ReaderLoopCommand>>
}

/// TcpReader event loop will be only connection reader/writer loop
/// we should't have connection accept functionality
impl Handler for TcpReader {
    type Timeout = ();
    type Message = ReaderLoopCommand;

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
                token: token,
                address: String::new()
            });
            return;
        }

        if events.is_readable() {
            if token == SERVER_TOKEN {
                // We shouldn't get Server token here
                return;
            } else {
                // finding connection here, reading some data and then registering to new events
                // if we got error during read process just reseting connection
                self.conns.find_connection_by_token(token)
                .and_then(|conn| conn.read_data_reader(event_loop))
                .unwrap_or_else(|_| {
                    self.net_chanel.send(NetLoopCmd {
                        cmd: LoopCommand::REMOVE_CONNECTION,
                        token: token,
                        address: String::new()
                    });
                })
            }
        }

        if events.is_writable() {
            // checking if we got write event for server or not
            // if it's true then just returning, because server can't have write event
            if token == SERVER_TOKEN {return;}

            // Writing data to available socket by token
            self.conns.find_connection_by_token(token)
            .and_then(|conn| conn.write_data_reader(event_loop))
            .unwrap_or_else(|_| {
                self.net_chanel.send(NetLoopCmd {
                    cmd: LoopCommand::REMOVE_CONNECTION,
                    token: token,
                    address: String::new()
                });
            })
        }
    }

    fn notify (&mut self, event_loop: &mut EventLoop<TcpReader>, cmd: ReaderLoopCommand) {
        match cmd.cmd {
            ReaderCommands::STOP_LOOP => {
                event_loop.shutdown();
            }
            ReaderCommands::HANDLE_CONNECTION => {
                let mut list = cmd.conn_socks;
                let conn = match list.pop() {
                    Some(c) => c,
                    None => return
                };
                match self.conns.insert_with(conn){
                    Some(token) => {
                        //if we got here then we successfully inserted connection
                        //now we need to register it
                        let st = match self.conns.find_connection_by_token(token) {
                            Ok(conn) => {
                                conn.register_reader(event_loop)
                            },
                            Err(e) => Err(e)
                        };

                        match st {
                            Ok(_) => {},
                            Err(_) => {
                                // if we got error during reregister process just removing connection from list
                                self.conns.remove_connection_by_token(token);
                            }
                        }
                    }
                    Nonde => {
                        Error::handle_error(ErrorCodes::NetworkTcpConnectionAccept, "Error inserting connection", "TcpReader Transfer Connection");
                    }
                };
            }
            ReaderCommands::WRITE_DATA => {
                if cmd.write_path[0] == Zero::zero() {
                    return;
                }

                for c in self.conns.conns.iter_mut() {
                    // if connection value is dividable to given path, then writing data to it
                    if cmd.write_path[0].mod_floor(&c.value) != Zero::zero() {
                        continue;
                    }

                    for i in 0..cmd.write_data.len() {
                        c.write(&cmd.write_data[i]);
                    }
                }
            }
        }
    }
}

impl TcpReader {
    pub fn new(net_chan: Sender<NetLoopCmd>) -> TcpReader {
        TcpReader {
            net_chanel: net_chan,
            conns: TcpConns::new(10),
            loop_channel: Vec::new(),
        }
    }

    pub fn run(net_chan: Sender<NetLoopCmd>) -> Sender<ReaderLoopCommand> {
        let mut event_loop: EventLoop<TcpReader> = EventLoop::new().ok().expect("Unable to create event loop for networking");
        let ret_chan = event_loop.channel();
        thread::spawn(move || {
            let mut r = TcpReader::new(net_chan);
            r.loop_channel.push(event_loop.channel());
            event_loop.run(&mut r);
        });

        return ret_chan;
    }

    /// Transfer connection to reader loop
    /// it is required to call event_loop.deregister for this connection before calling this function, for thread safety
    pub fn transfer_connection_raw(&mut self, token: Token, event_loop: &mut EventLoop<TcpReader>) -> io::Result<()> {
        //extracting network pointer

        self.conns.find_connection_by_token(token)
        .and_then(|conn| conn.register_reader(event_loop))
        .unwrap_or_else(|_| {
            Error::handle_error(ErrorCodes::NetworkErrorEvent, "Unable to transfer connection to Reader loop", "Reader Transfer connection");
            self.net_chanel.send(NetLoopCmd {
                cmd: LoopCommand::REMOVE_CONNECTION,
                token: token,
                address: String::new()
            });
        });

        Ok(())
    }
}
