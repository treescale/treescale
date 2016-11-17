#![allow(dead_code)]
#![allow(unreachable_code)]
extern crate mio;
extern crate num;
extern crate num_cpus;
extern crate byteorder;

use network::tcp::{TcpConnection, TcpReaderCommand, TcpReader, TcpReaderCMD};
use self::mio::{Token, Poll, Ready, PollOpt, Events};
use self::mio::channel::{Sender, Receiver, channel};
use self::mio::tcp::{TcpStream, TcpListener};
use std::sync::Arc;
use self::num::bigint::{BigInt, Sign};
use self::num::Zero;
use std::ops::Mul;
use std::thread;
use std::io::{Result, ErrorKind, Error, Cursor};
use self::byteorder::{BigEndian, ReadBytesExt};
use std::collections::BTreeMap;
use std::os::unix::io::AsRawFd;
use std::str::FromStr;
use std::net::SocketAddr;
use std::ops::Rem;

const TCP_NET_CHANNEL_TOKEN: Token = Token(0);

pub enum TcpNetworkCMD {
    ConnectionClosed,
    HandleNewData,
}

pub struct TcpNetworkCommand {
    pub code: TcpNetworkCMD,
    pub token: Token,
    pub data: Vec<Arc<Vec<u8>>>
}

pub struct TcpNetwork {
    // base connections vector for keeping full networking connections
    pub connections: Vec<TcpConnection>,
    // map for keeping vector keys based on connections
    // beacuse we are getting events based on connection keys
    connection_keys: BTreeMap<Token, usize>,

    // current Node prime value
    pub value: BigInt,
    pub value_square: BigInt,

    // Tcp Reader channels for sending commands
    pub reader_channels: Vec<Sender<TcpReaderCommand>>,
    pub readers_count: usize,
    // index for picking up reader using Round Rubin
    readers_index: usize,

    // token for current Node
    // this would be used for client connection handshake process
    pub token: String,

    // channel for sending commands to current TcpNetwork
    pub channel_sender: Sender<TcpNetworkCommand>,
    channel_receiver: Receiver<TcpNetworkCommand>,

    // TcpServer address for handling connections
    server_address: String,

    // base poll object for handling loop events
    poll: Poll
}

impl TcpNetwork {
    /// Making new networking object with given parameters
    pub fn new(token: String, value: BigInt, readers_count: usize, server_address: String) -> TcpNetwork {
        let mut rc = readers_count;
        if readers_count == 0 {
            rc = num_cpus::get();
        }

        let (s, r) = channel::<TcpNetworkCommand>();

        TcpNetwork {
            connections: Vec::new(),
            value: value.clone(),
            value_square: value.clone().mul(value.clone()),
            reader_channels: Vec::with_capacity(rc),
            readers_count: rc,
            token: token,
            channel_sender: s,
            channel_receiver: r,
            poll: Poll::new().unwrap(),
            connection_keys: BTreeMap::new(),
            readers_index: 0,
            server_address: server_address
        }
    }

    #[inline(always)]
    pub fn channel(&self) -> Sender<TcpNetworkCommand> {
        self.channel_sender.clone()
    }

    #[inline(always)]
    fn add_new_connection(&mut self, socket: TcpStream) {
        let token = Token(socket.as_raw_fd() as usize);
        if token == TCP_NET_CHANNEL_TOKEN {
            return;
        }

        // getting reader with basic round rubin for transfering connection to it
        if self.readers_index >= self.readers_count {
            self.readers_index = 0;
        }

        // sending connection socket to reader
        match self.reader_channels[self.readers_index].send(TcpReaderCommand {
            code: TcpReaderCMD::HandleNewConnection,
            socket: vec![socket],
            token: vec![token],
            data: Vec::new()
        }) {
            Ok(_) => {},
            // if we got error during chanel send, then we can't add connection on this moment
            Err(_) => return
        }

        // saving connection to our connection list
        self.connections.push(TcpConnection::new(self.reader_channels[self.readers_index].clone(), token));

        // moving to the next reader
        self.readers_index += 1;
    }

    #[inline(always)]
    fn path_from_data(data: Arc<Vec<u8>>) -> Result<BigInt> {
        let mut path_len_buf: Vec<u8> = Vec::with_capacity(4);
        // first 4 bytes should be a length of a path string
        for i in 0..4 {
            path_len_buf.push(data[i]);
        }

        let mut rdr = Cursor::new(path_len_buf);
        let path_len =  match rdr.read_u32::<BigEndian>() {
            Ok(s) => s as usize,
            Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Unable to parse path length from given data"))
        };

        let mut path_data: Vec<u8> = Vec::with_capacity(path_len);
        for i in 4..path_len {
            path_data.push(data[i]);
        }


        Ok(BigInt::from_bytes_be(Sign::Plus, path_data.as_slice()))
    }

    /// Running networking with specific readers and server threads
    pub fn run(&mut self) -> Result<()> {
        // making TcpListener for making server socket
        let addr = match SocketAddr::from_str(self.server_address.as_str()) {
            Ok(a) => a,
            Err(_) => return Err(Error::new(ErrorKind::AddrNotAvailable, "Unable to parse given server address"))
        };

        let server = TcpListener::bind(&addr).unwrap();
        let server_token = Token(server.as_raw_fd() as usize);

        // registering channel for receiveing commands
        match self.poll.register(&self.channel_receiver, server_token, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        // registering tcp server for accepting connections
        match self.poll.register(&server, TCP_NET_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }

        // Making readers and starting them, by keeping their channels
        for _ in 0..self.readers_count {
            let mut reader = TcpReader::new(self.channel_sender.clone());
            self.reader_channels.push(reader.channel());
            thread::spawn(move || {
                let _ = reader.run();
            });
        }

        let mut events: Events = Events::with_capacity(1000);

        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.into_iter() {
                let token = event.token();
                if token == TCP_NET_CHANNEL_TOKEN {
                    // trying to get commands while there is available data
                    loop {
                        match self.channel_receiver.try_recv() {
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

                if token == server_token {
                    loop {
                        match server.accept() {
                            Ok((sock, _)) => {
                                self.add_new_connection(sock);
                            }
                            // if we got error on server accept process
                            // we need to break accept loop and wait until new connections
                            // would be available in event loop
                            Err(_) => break
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn notify(&mut self, cmd: &mut TcpNetworkCommand) {
        match cmd.code {
            TcpNetworkCMD::ConnectionClosed => {
                if !self.connection_keys.contains_key(&cmd.token) {
                    return;
                }

                // removing connection object from our list if connection closed
                let i = self.connection_keys[&cmd.token];
                self.connections.remove(i);
            }

            TcpNetworkCMD::HandleNewData => {
                while !cmd.data.is_empty() {
                    let data = cmd.data.remove(0);
                    let path = match TcpNetwork::path_from_data(data.clone()) {
                        Ok(p) => p,
                        // if we can't get path from data, just moving forward
                        Err(_) => continue
                    };

                    for i in 0..self.connections.len() {
                        // ignoring connection who sent this data
                        if self.connections[i].socket_token == cmd.token {
                            continue;
                        }

                        // if connection value is dividable to path
                        // writing data to connection
                        if path.clone().rem(&self.connections[i].value) == BigInt::zero() {
                            self.connections[i].write_data(data.clone());
                        }
                    }
                }
            }
        }
    }
}
