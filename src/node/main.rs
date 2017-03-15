#![allow(dead_code)]
extern crate mio;
extern crate threadpool;

use self::mio::Poll;
use self::mio::channel::{channel, Sender, Receiver};
use self::threadpool::ThreadPool;
use self::mio::tcp::{TcpListener};

use event::{EventCallback, EventCommand};
use network::{NetworkCommand, Connection
              , TcpHandlerCMD, TcpHandlerCommand
              , Slab, TcpConnection};

use std::collections::BTreeMap;

pub struct Node {
    /// Callbacks map for handling it in EventHandler trait
    pub callbacks: BTreeMap<String, Vec<EventCallback>>,
    // channels for handling events from networking
    pub event_sender_chan: Sender<EventCommand>,
    pub event_receiver_chan: Receiver<EventCommand>,

    /// Members for Network trait
    pub connections: BTreeMap<String, Connection>,
    pub net_sender_chan: Sender<NetworkCommand>,
    pub net_receiver_chan: Receiver<NetworkCommand>,

    /// TCP networking params
    pub net_tcp_reader_sender_chan: Vec<Sender<TcpHandlerCommand>>,
    pub net_tcp_writer_sender_chan: Vec<Sender<TcpHandlerCommand>>,
    // index for load balancing over TCP Reader and Writer channels
    pub net_tcp_handler_index: usize,
    // TCP server socket
    pub net_tcp_server: TcpListener,
    // keeping just a simple TcpConnection as a pending connection
    pub net_tcp_pending_connections: Slab<TcpConnection>,

    /// POLL service for this node thread event loop
    pub poll: Poll,

    /// Thread Pool for making background precessing tasks
    pub thread_pool: ThreadPool,
}


impl Node {
}