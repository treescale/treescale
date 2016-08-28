extern crate mio;
extern crate num;

use mio::{Token, EventLoop, EventSet, PollOpt};
use mio::tcp::TcpStream;
use self::num::bigint::{BigInt, Sign};
use network::tcp_net::TcpNetwork;
use network::tcp_reader::TcpReader;
use error::error::Error;
use error::codes::ErrorCodes;
use std::io;
use mio::util::Slab;

pub struct TcpConns {
    pub conns: Slab<TcpConnection>
}

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
    interest: EventSet
}

impl TcpConnection {
    pub fn new(sock: TcpStream, token: Token, from_server: bool) -> TcpConnection {
        TcpConnection {
            sock: sock,
            key: token,
            token: String::new(),
            value: BigInt::new(Sign::Plus, vec![0]),
            from_server: from_server,
            interest: EventSet::hup()
        }
    }

    // TODO: implement Write functionality with write Queue
    /// Write function for this connection, it will handle byte data and will queue it inside list
    /// and when connection will be ready to send it from MIO loop it will make it from "writable" function
    pub fn write(&mut self) -> io::Result<()> {
        // inserting write interest to handle write event when loop will be ready
        self.interest.insert(EventSet::writable());
        Ok(())
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
        self.reregister_net(event_loop)
    }

    pub fn read_data_reader(&mut self, event_loop: &mut EventLoop<TcpReader>) -> io::Result<()> {
        self.reregister_reader(event_loop)
    }

    // TODO: implement this function for write functionality
    /// This function will be triggered from event loop, when our socket will be ready for writing
    /// We will write all queued data at once for giving more performance
    pub fn write_data_net(&mut self, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()> {
        self.reregister_net(event_loop)
    }

    pub fn write_data_reader(&mut self, event_loop: &mut EventLoop<TcpReader>) -> io::Result<()> {
        self.reregister_reader(event_loop)
    }
}
