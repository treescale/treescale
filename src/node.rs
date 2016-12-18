#![allow(dead_code)]
extern crate mio;

use self::mio::channel::Sender;
use network::tcp::{TcpNetwork, TcpNetworkCommand, TcpNetworkCMD};
use event::{Event, EventHandler, EVENT_ON_PENDING_CONNECTION, EVENT_ON_CONNECTION_CLOSE};
use std::thread;
use std::sync::Arc;
use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;
use self::mio::tcp::TcpStream;
use std::str::FromStr;

pub struct Node {
    // main event handler loop for current node
    event_handler: Vec<EventHandler>,
    // tcp network handler for current node
    tcp_net_channel: Sender<TcpNetworkCommand>,

    // token and value for current node
    token: String,
    value: String,

    // server port for binding TCP Listener
    server_address: String,
}

impl Node {
    // making new node and starting networking
    pub fn new(token: &str, value: &str, address: &str) -> Node {
        let ev = EventHandler::new();
        let mut tcp_net = TcpNetwork::new(String::from(token), String::from(value), ev.channel());
        let addr_str = String::from(address);
        let node = Node {
            event_handler: vec![ev],
            tcp_net_channel: tcp_net.channel(),
            server_address: String::from(address),
            token: String::from(token),
            value: String::from(value)
        };

        thread::spawn(move || {
            tcp_net.run(addr_str.as_str(), 3);
        });

        node
    }

    // starting event handler loop for getting events
    pub fn run(&mut self) {
        let ev = self.event_handler.pop().unwrap();
        ev.run(self);
    }

    // just a shortcut function for adding events
    pub fn on(&mut self, name: &str, callback: Box<Fn(Arc<Event>, &mut Node)>) {
        let mut ev = self.event_handler.pop().unwrap();
        ev.on(name, callback);
        self.event_handler.push(ev);
    }

    pub fn off(&mut self, name: &str) {
        let mut ev = self.event_handler.pop().unwrap();
        ev.remove(name);
        self.event_handler.push(ev);
    }

    // functions with specific events
    pub fn on_pending_conn(&mut self, callback: Box<Fn(Arc<Event>, &mut Node)>) {
        let mut ev = self.event_handler.pop().unwrap();
        ev.on(EVENT_ON_PENDING_CONNECTION, callback);
        self.event_handler.push(ev);
    }

    pub fn on_conn_close(&mut self, callback: Box<Fn(Arc<Event>, &mut Node)>) {
        let mut ev = self.event_handler.pop().unwrap();
        ev.on(EVENT_ON_CONNECTION_CLOSE, callback);
        self.event_handler.push(ev);
    }

    pub fn connect(&mut self, address: &str) -> Result<()> {
        // making TcpListener for making server socket
        let addr = match SocketAddr::from_str(address) {
            Ok(a) => a,
            Err(_) => return Err(Error::new(ErrorKind::AddrNotAvailable, "Unable to make address lookup"))
        };

        let _ = self.tcp_net_channel.send(TcpNetworkCommand {
            cmd: TcpNetworkCMD::HandleClientConnection,
            socket: vec![match TcpStream::connect(&addr) {
                Ok(s) => s,
                Err(e) => return Err(e)
            }],
            token: vec![],
            event: vec![]
        });

        Ok(())
    }

    pub fn accept(&self, token: String) {
        let _ = self.tcp_net_channel.send(TcpNetworkCommand {
            cmd: TcpNetworkCMD::AcceptPendingConnection,
            socket: vec![],
            token: vec![token],
            event: vec![]
        });
    }

    pub fn emit(&self, name: &str, path: &str, data: &str) {
        let _ = self.tcp_net_channel.send(TcpNetworkCommand {
            cmd: TcpNetworkCMD::EmitEvent,
            socket: vec![],
            token: vec![],
            event: vec![Event{
                name: String::from(name),
                from: self.token.clone(),
                target: String::from(path),
                data: String::from(data),
                path: String::from(path),
                public_data: String::new()
            }]
        });
    }
}
