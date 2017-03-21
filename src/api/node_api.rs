#![allow(dead_code)]
extern crate mio;
extern crate threadpool;
extern crate num_cpus;
extern crate slab;

use std::collections::BTreeMap;
use std::error::Error;
use std::process;

use helper::conn::Connection;
use helper::tcp_conn::TcpConnection;
use helper::Log;
use api::ApiTcpNetwork;
use event::Event;

use self::mio::{Poll, Events, Token};
use self::threadpool::ThreadPool;

pub type Slab<T> = slab::Slab<T, Token>;
const SLAB_CONNECTIONS_ALLOCATION: usize = 500;

/// Main structure to handle API client functionality
pub struct NodeApi {
    /// Token for this API
    pub token: String,
    /// API version for this API
    pub api_version: u32,
    /// Connections for this API client
    pub connections: BTreeMap<String, Connection>,

    /// POLL service for this node thread event loop
    pub poll: Poll,
    /// Thread Pool for making background precessing tasks
    pub thread_pool: ThreadPool,
    /// keeping TCP connections as a list of live sockets
    pub tcp_connections: Slab<TcpConnection>,
}

impl NodeApi {
    /// Making new API client based on given Token
    pub fn new(token: String) -> NodeApi {
        NodeApi {
            token: token,
            api_version: 1,
            connections: BTreeMap::new(),
            poll: match Poll::new() {
                Ok(p) => p,
                Err(e) => {
                    Log::error("Unable to make POLL service for API client", e.description());
                    process::exit(1);
                }
            },
            thread_pool: ThreadPool::new(num_cpus::get()),
            tcp_connections: Slab::with_capacity(SLAB_CONNECTIONS_ALLOCATION)
        }
    }

    /// Starting API service Event Loop
    pub fn start(&mut self) {
        // making events only with 1K events per iteration for not getting a lot of memory
        // anyway it is API client so it wouldn't consume so match
        let mut events: Events = Events::with_capacity(1000);

        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.iter() {
                let (token, kind) = (event.token(), event.kind());

                // if this is a TCP connection event
                if self.tcp_ready(token, kind) {
                    continue;
                }
            }
        }
    }

    /// Removing specific Identity from Connection
    /// if this is the last identity, then all connection would be removed from the list
    #[inline]
    pub fn remove_identity(&mut self, token: String, socket_token: Token) {

    }

    pub fn handle_event(&mut self, event: Event) {

    }
}