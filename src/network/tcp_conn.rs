extern crate mio;
extern crate num;
extern crate byteorder;

use mio::{Token, EventLoop, EventSet, PollOpt};
use mio::tcp::TcpStream;
use self::num::bigint::{BigInt, Sign};
use network::tcp_net::TcpNetwork;
use network::tcp_reader::TcpReader;
use error::error::Error;
use error::codes::ErrorCodes;
use std::io;
use mio::util::Slab;
use std::io::{Read, Write};
use self::byteorder::{BigEndian, ByteOrder};
use std::sync::RwLock;
use std::rc::Rc;

pub struct TcpConns {
    pub conns: Slab<TcpConnection>
}

const CONN_READ_CHUNCK_LENGTH: usize = 5120;

impl TcpConns {
    pub fn new(max: usize) -> TcpConns {
        TcpConns {
            conns: Slab::new_starting_at(Token(2), max)
        }
    }

    /// Find a connection in the slab using the given token.
    pub fn find_connection_by_token<'a>(&'a mut self, token: Token) -> io::Result<&'a mut TcpConnection> {
        if self.conns.contains(token) {
            Ok(&mut self.conns[token])
        }
        else {
            Err(io::Error::new::<&str>(io::ErrorKind::NotFound, "connection with token not found"))
        }
    }

    pub fn remove_connection_by_token(&mut self, token: Token) {
        if self.conns.contains(token) {
            self.conns.remove(token);
        }
    }

    pub fn insert_with(&mut self, sock: TcpStream) -> Option<Token> {
        self.conns.insert_with(|token| {
            TcpConnection::new(sock, token, true)
        })
    }
}



pub struct TcpConnection {
    from_server: bool,
    token: String,
    value: BigInt,
    pub sock: TcpStream,

    // This fileds is for running connection inside event loop
    // this will be token for handling MIO connection loop
    key: Token,
    // Set of event interesting for us for this connection
    interest: EventSet,

    // variables for keeping received and remaining data values
    data_len: u32,
    // just allocating it here for not making 4 byte allocation on every data receiving
    data_len_bytes: Vec<u8>,
    data_received_len: u32,
    data: Vec<Vec<u8>>,

    // Write data for keeping some queue of data
    // this will help save data until it would be ready to write
    // using Read Write lock for cross thread usage
    write_queue: RwLock<Vec<Rc<Vec<u8>>>>
}


impl TcpConnection {
    pub fn new(sock: TcpStream, token: Token, from_server: bool) -> TcpConnection {
        TcpConnection {
            sock: sock,
            key: token,
            token: String::new(),
            value: BigInt::new(Sign::Plus, vec![0]),
            from_server: from_server,
            interest: EventSet::hup(),
            data_len: 0,
            data_received_len: 0,
            data: vec![],  // making empty vector
            data_len_bytes: vec![0; 4], // 4 bytes for BigEndian number
            write_queue: RwLock::new(Vec::new())
        }
    }

    /// Write function for this connection, it will handle byte data and will queue it inside list
    /// and when connection will be ready to send it from MIO loop it will make it from "writable" function
    pub fn write(&mut self, data: &mut Rc<Vec<u8>>) -> io::Result<()> {
        // Getting write access and writing data to it
        // this will lock data until it would be fully written and will end the scope
        {
            let mut locker_data = self.write_queue.write().unwrap();
            locker_data.push(data.clone());
        }

        // inserting write interest to handle write event when loop will be ready
        self.interest.insert(EventSet::writable());
        Ok(())
    }

    /// Main function to read data from socket
    /// this function will be called inside event loop "read_data" event
    /// Basic logic of data API is to read first 4 bytes as a BigEndian integer, which would be the length of remaining data
    /// then just read that remaining data and return from function as a vector of bytes
    ///
    /// Main thing is that until data is collected we will keep remaining buffer in "self"
    /// but allocation process would be chunck by chunck as we receiving
    /// function will return "true" if we collected all remaining data
    pub fn read(&mut self) -> io::Result<(bool, Vec<u8>)> {
        let mut sock = &mut self.sock;
        // if our data is reseted we need to start reading new data
        if self.data.len() == 0 && self.data_len == 0 {
            match sock.take(4).read(&mut self.data_len_bytes) {
                Ok(n) => {
                    if n != 4 {
                        return Ok((false, vec![]));
                    }
                },
                Err(e) => {
                    Error::handle_error(ErrorCodes::TcpConnectionRead, "Error while trying to read from tcp connection", "Tcp Connection Read");
                    return Ok((false, vec![]));
                }
            }

            self.data_len = BigEndian::read_u32(&self.data_len_bytes);
            // if we don' have data to read just returning
            if self.data_len <= 0 {
                self.data_len = 0;
                return Ok((false, vec![]));
            }

            self.data_received_len = 0;

            // Adding new data vector to read
            // not allocating because we will append to it chunck by chunck
            self.data.push(vec![]);
        }

        let mut read_data = vec![0; CONN_READ_CHUNCK_LENGTH];

        loop {
            match sock.read(&mut read_data) {
                Ok(n) => {
                    if n > 0 {
                        read_data.split_off(n);
                        self.data[0].append(&mut read_data);
                        read_data.clear();  // clearing data right after appending it
                        self.data_received_len += n as u32;
                    }

                    // if we got less bytes than we ready to read then our data is completed on this EventLoop cycle
                    if n < CONN_READ_CHUNCK_LENGTH {
                        break;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // if we got here then we finished data reading process for this loop cycle

        // if we got more data than expected by API, then just clearing and returning
        if self.data_received_len > self.data_len {
            self.data[0].clear();
            self.data.clear();
            self.data_len = 0;
            self.data_received_len = 0;
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Wrong API for recieved data, seems need to close connection"));
        }

        if self.data_received_len == self.data_len {
            match self.data.pop() {
                Some(ret_data) => {
                    self.data.clear();
                    self.data_len = 0;
                    self.data_received_len = 0;
                    return Ok((true, ret_data));
                }
                None => {
                    return Ok((false, vec![]));
                }
            }

        }

        Ok((false, vec![]))
    }

    /// Register connection to networking event loop
    pub fn register_net(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()> {
        // Adding reading interest for this connection
        self.interest.insert(EventSet::readable());

        event_loop.register(
            &self.sock,
            self.key,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e|{
            Error::handle_error(ErrorCodes::TcpConnectionRegisterReadInterest, "Error while trying to register tcp connection read interest", "Tcp Connection Register");
            Err(e)
        })
    }

    pub fn register_reader(&mut self, event_loop: &mut EventLoop<TcpReader>) -> io::Result<()> {
        // Adding reading interest for this connection
        self.interest.insert(EventSet::readable());

        event_loop.register(
            &self.sock,
            self.key,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e|{
            Error::handle_error(ErrorCodes::TcpConnectionRegisterReadInterest, "Error while trying to register tcp connection read interest", "Tcp Connection Register");
            Err(e)
        })
    }

    /// Reregistering existing connection to get more events which could be queued by MIO
    pub fn reregister_net(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()> {
        event_loop.reregister(
            &self.sock,
            self.key,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            Error::handle_error(ErrorCodes::TcpConnectionReRegisterReadInterest, "Error while trying to ReRegister tcp connection read interest", "Tcp Connection ReRegister");
            Err(e)
        })
    }

    pub fn reregister_reader(&mut self, event_loop: &mut EventLoop<TcpReader>) -> io::Result<()> {
        event_loop.reregister(
            &self.sock,
            self.key,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            Error::handle_error(ErrorCodes::TcpConnectionReRegisterReadInterest, "Error while trying to ReRegister tcp connection read interest", "Tcp Connection ReRegister");
            Err(e)
        })
    }


    // TODO: implement connection read logic here
    /// We will read connection data here using speficic TreeScale API
    /// this function will be triggered when networking event loop will be ready to read some data from connection
    pub fn read_data_net(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()> {
        match self.read() {
            Ok((done, data)) => {
                if done {
                    // Handle Data here
                    println!("{:?}", data);
                }
            }
            Err(e) => return Err(e)
        }
        self.reregister_net(event_loop)
    }

    pub fn read_data_reader(&mut self, event_loop: &mut EventLoop<TcpReader>) -> io::Result<()> {
        match self.read() {
            Ok((done, data)) => {
                if done {
                    // Handle Data here
                    println!("{:?}", data);
                }
            }
            Err(e) => return Err(e)
        }

        self.reregister_reader(event_loop)
    }

    pub fn flush_write_queue(&mut self) -> io::Result<()> {
        let mut queue_locked = self.write_queue.write().unwrap();

        loop {
            match queue_locked.pop() {
                Some(data_rc) => {
                    // Writing data len, den full data with it
                    // if we will try to combine first 4 bytes then all data, we will need to allocate new space
                    // so trying without new allocation
                    {
                        let mut write_data_len = vec![0; 4];
                        BigEndian::write_u32(&mut write_data_len, data_rc.len() as u32);
                        match self.sock.write_all(write_data_len.as_slice()) {
                            Ok(()) => {}
                            Err(e) => return Err(e)
                        }
                    }
                    match self.sock.write_all(data_rc.as_slice()) {
                        Ok(()) => {}
                        Err(e) => return Err(e)
                    };
                }
                None => {
                    break;
                }
            }
        }

        Ok(())
    }

    /// This function will be triggered from event loop, when our socket will be ready for writing
    /// We will write all queued data at once for giving more performance
    pub fn write_data_net(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()> {
        match self.flush_write_queue() {
            Ok(()) => {}
            Err(e) => return Err(e)
        };

        self.reregister_net(event_loop)
    }

    pub fn write_data_reader(&mut self, event_loop: &mut EventLoop<TcpReader>) -> io::Result<()> {
        match self.flush_write_queue() {
            Ok(()) => {}
            Err(e) => return Err(e)
        };

        self.register_reader(event_loop)
    }
}
