extern crate mio;
use std::io;
use std::io::{Error, ErrorKind};
use mio::{EventLoop, Token};
use mio::tcp::{TcpStream};
use std::net::SocketAddr;
use std::str::FromStr;
use network::tcp_net::{TcpNetwork, NetLoopCmd, LoopCommand};

pub trait TcpClient {
    /// with this function we will send notify from channel to Networking loop
    /// because we expecting to call this function from outiside threads
    fn connect(&mut self, address: &str) -> io::Result<()>;

    /// this function is for internal loop call, here we will have real connection functionality
    fn connect_raw(&mut self, address: &str, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()>;
}

impl TcpClient for TcpNetwork {
    fn connect(&mut self, address: &str) -> io::Result<()> {
        let mut send_chan = self.event_loop.channel();
        match send_chan.send(NetLoopCmd {
            token: Token(0), // we don't care about token here
            cmd: LoopCommand::CLIENT_CONNECT,
            address: String::from(address)
        }) {
            Ok(_) =>   return Ok(()),
            Err(e) => return Err(Error::new(ErrorKind::Interrupted, "unable to send channel command to Networking event loop"))
        };
    }

    fn connect_raw(&mut self, address: &str, event_loop: &mut EventLoop<TcpNetwork>) -> io::Result<()> {
        let addr = match SocketAddr::from_str(address) {
            Ok(a) => a,
            Err(e) => return Err(Error::new(ErrorKind::Interrupted, e))
        };

        let sock = match TcpStream::connect(&addr) {
            Ok(s) => s,
            Err(e) => return Err(e)
        };

        match self.connections.insert_with(sock) {
            Some(token) => {
                //if we got here then we successfully inserted connection
                //now we need to register it
                match self.connections.find_connection_by_token(token) {
                    Ok(conn) => {
                        match conn.register_net(event_loop) {
                            Ok(_) => {},
                            Err(_) => {
                                // if we got error during reregister process just removing connection from list
                                // self.connections.remove(token);
                            }
                        }
                    },
                    Err(e) => { }
                };
            }
            None => return Err(Error::new(ErrorKind::Interrupted, "error inserting connection to list"))
        };

        Ok(())
    }
}
