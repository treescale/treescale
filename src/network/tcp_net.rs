extern crate mio;

use mio::{Handler, EventLoop, Token, EventSet, Sender, PollOpt};
use mio::tcp::{TcpListener, TcpStream};
use network::tcp_server::{TcpServer, SERVER_TOKEN};
use network::tcp_conn::{TcpConns};
use network::tcp_reader::{TcpReader, ReaderLoopCommand, ReaderCommands};
use error::error::Error;
use error::codes::ErrorCodes;
use std::sync::{Arc};
use network::tcp_client::TcpClient;
use std::thread;
use std::sync::mpsc::channel;
use std::net::SocketAddr;
use std::str::FromStr;
use std::io::Write;
use std::io;

pub const INVALID_TOKEN: Token = Token(0);
pub const MAX_CONNECTIONS: usize = 100000000;

pub enum LoopCommand {
    STOP_LOOP,
    REMOVE_CONNECTION,
    ACCEPT_CONNECTION,
    CLIENT_CONNECT,
    TRANSFER_CONNECTION,
}

pub struct NetLoopCmd {
    pub cmd: LoopCommand,
    pub token: Token,
    // this address would be used in client connection command
    pub address: String
}

pub struct TcpNetwork {
    pub connections: TcpConns,
    pub is_api: bool,
    pub loop_channel: Vec<Sender<NetLoopCmd>>,

    // main server socket
    pub server_sock: TcpListener,
    pub server_address: String,

    // keeping TcpReaders for transfering connection to read process
    pub readers: Vec<Sender<ReaderLoopCommand>>,
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
            LoopCommand::CLIENT_CONNECT => {
                match self.connect_raw(cmd.address.as_str(), event_loop) {
                    Ok(()) => {}
                    Err(e) => Error::handle_error(ErrorCodes::TcpClientConnectionFail, "Error while trying to connect to given address", "Networking TcpClient Ready State")
                }
            }
            LoopCommand::TRANSFER_CONNECTION => {
                self.transfer_connection(cmd.token);
            }
        }
    }
}

impl TcpNetwork{

    pub fn new(server_address: &str, is_api: bool, readers_count: usize) -> TcpNetwork {
        let addr = SocketAddr::from_str(server_address).unwrap();

        TcpNetwork {
            connections: TcpConns::new(10),
            server_sock: TcpListener::bind(&addr).ok().expect("Error binding server"),
            is_api: is_api,
            server_address: String::from(server_address),
            readers_index: readers_count,
            readers: Vec::new(),
            loop_channel: Vec::new()
        }
    }

    /// This function will start event loop and will register server if it's exists
    pub fn run(server_address: &str, is_api: bool, readers_count: usize) -> (Sender<NetLoopCmd>, Vec<Sender<ReaderLoopCommand>>) {
        let mut sv = String::from(server_address);
        let (chan_sender, chan_reader) = channel();
        let mut ret_reader_chans: Vec<Sender<ReaderLoopCommand>> = Vec::new();
        let (chan_s_readers, chan_r_readers) = channel();
        thread::spawn(move || {
            let mut net = TcpNetwork::new(sv.as_str(), is_api, readers_count);
            let mut event_loop: EventLoop<TcpNetwork> = EventLoop::new().ok().expect("Unable to create event loop for networking");

            chan_sender.send(event_loop.channel());
            net.register_server(&mut event_loop);
            net.loop_channel.push(event_loop.channel());

            for i in 0..readers_count {
                let reader_chan = TcpReader::run(event_loop.channel());
                net.readers.push(reader_chan.clone());
                ret_reader_chans.push(reader_chan.clone());
            }

            chan_s_readers.send(ret_reader_chans);

            event_loop.run(&mut net);
        });

        let ret_net_chan = chan_reader.recv().unwrap();
        let ret_readers_chan = chan_r_readers.recv().unwrap();

        return (ret_net_chan, ret_readers_chan);
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

    fn transfer_connection(&mut self, token: Token) {
        if self.readers_index >= self.readers.len() {
            self.readers_index = 0;
        }

        let mut conn = Vec::new();
        match self.connections.remove_connection_by_token(token) {
            Some(c) => {
                conn.push(c.sock);
                self.readers[self.readers_index].send(ReaderLoopCommand{
                    cmd: ReaderCommands::HANDLE_CONNECTION,
                    conn_socks: conn,
                    write_data: Vec::new(),
                    write_path: Vec::new()
                });
            },
            None => {}
        };

        self.readers_index += 1;
    }
}
