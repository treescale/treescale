extern crate mio;
extern crate num;

use network::tcp::{TcpConnValue, TcpConn};
use event::{EventHandlerCommand, Event, EventHandlerCMD, EVENT_ON_CONNECTION_CLOSE, EVENT_ON_CONNECTION};
use std::sync::{Arc, RwLock};
use self::mio::channel::{Sender, Receiver, channel};
use self::mio::{Token, Poll, Ready, PollOpt, Events};
use self::num::{BigInt, Zero};
use std::process;
use std::collections::BTreeMap;
use std::io::{ErrorKind, Read};
use std::str::FromStr;

const RECEIVER_CHANNEL_TOKEN: Token = Token(1);
const READER_BUFFER_SIZE: usize = 65000;

pub enum TcpReaderCMD {
    HandleConnection,
    WriteDataToConn,
    WriteDataWithPath
}

pub struct TcpReaderCommand {
    pub cmd: TcpReaderCMD,
    pub conn_value: Vec<TcpConnValue>,
    pub conn: Vec<TcpConn>,
    pub data: Vec<Arc<Vec<u8>>>,
    pub socket_token: Vec<Token>,
    pub tokens: Vec<String>,
    pub event: Vec<Event>
}

pub struct TcpReader {
    connections: Arc<RwLock<Vec<TcpConnValue>>>,
    reader_conns: BTreeMap<Token, TcpConn>,

    // prime value for current node
    // we will use this for checking event trigger
    current_value: BigInt,

    // reader sender channels
    sender_channel: Sender<TcpReaderCommand>,
    receiver_channel: Receiver<TcpReaderCommand>,

    // channel for triggering events from networking
    event_handler_channel: Sender<EventHandlerCommand>,

    // Vector of channels for readers including current one
    pub reader_channels: Vec<Sender<TcpReaderCommand>>,
    pub reader_index: usize,

    // allocated buffer for batch reading from connection
    readable_buffer: Vec<u8>,

    poll: Poll,
    big_zero: BigInt
}

impl TcpReader {
    pub fn new(connections: Arc<RwLock<Vec<TcpConnValue>>>, event_handler: Sender<EventHandlerCommand>, value: BigInt) -> TcpReader {
        let (s, r) = channel::<TcpReaderCommand>();
        TcpReader {
            connections: connections,
            sender_channel: s,
            receiver_channel: r,
            reader_channels: vec![],
            reader_index: 0,
            poll: match Poll::new() {
                Ok(p) => p,
                Err(e) => {
                    warn!("Unable to create Poll service from TcpReader -> {}", e);
                    process::exit(1);
                }
            },
            big_zero: Zero::zero(),
            reader_conns: BTreeMap::new(),
            readable_buffer: vec![0; READER_BUFFER_SIZE],
            event_handler_channel: event_handler,
            current_value: value
        }
    }

    pub fn channel(&self) -> Sender<TcpReaderCommand> {
        self.sender_channel.clone()
    }

    pub fn run(&mut self) {
        match self.poll.register(&self.receiver_channel, RECEIVER_CHANNEL_TOKEN, Ready::readable(), PollOpt::edge()) {
            Ok(_) => {},
            Err(e) => {
                warn!("Unable to register receiver channel to Poll service for Reader -> {}", e);
                return;
            }
        }

        // making events for handling 5K events at once
        let mut events: Events = Events::with_capacity(5000);
        loop {
            let event_count = self.poll.poll(&mut events, None).unwrap();
            if event_count == 0 {
                continue;
            }

            for event in events.into_iter() {
                let token = event.token();
                if token == RECEIVER_CHANNEL_TOKEN {
                    // trying to get commands while there is available data
                    loop {
                        match self.receiver_channel.try_recv() {
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

                let kind = event.kind();

                if kind.is_error() || kind.is_hup() {
                    // if this error on connection, then we need to close it
                    self.close_connection(token);
                    continue;
                }

                if kind.is_readable() {
                    self.readable(token);
                    continue;
                }

                if kind.is_writable() {
                    self.writable(token);
                    continue;
                }
            }
        }
    }

    #[inline(always)]
    fn notify(&mut self, command: &mut TcpReaderCommand) {
        match command.cmd {
            TcpReaderCMD::HandleConnection => {
                let conn = match command.conn.pop() {
                    Some(c) => c,
                    None => return
                };

                let conn_value = match command.conn_value.pop() {
                    Some(c) => c,
                    None => return
                };

                // keeping this for event trigger
                let conn_value_token = conn_value.token.clone();

                match self.poll.reregister(&conn.socket, conn.socket_token, Ready::writable(), PollOpt::edge()) {
                    Ok(_) => {},
                    Err(e) => {
                        warn!("Unable to register connection from reader, closing it -> {}", e);
                        return;
                    }
                }

                // locking connections as writable for inserting received connection
                {
                    let mut conns_v = match self.connections.write() {
                        Ok(c) => c,
                        Err(e) => {
                            warn!("Unable to set writable lock for global connections list -> {}", e);
                            return;
                        }
                    };

                    conns_v.push(conn_value);
                }

                // inserting connection to the list of current reader
                self.reader_conns.insert(conn.socket_token, conn);

                let mut ev = Event::default();
                ev.name = String::from(EVENT_ON_CONNECTION);
                ev.from = conn_value_token;

                // triggering event about new accepted connection
                let _ = self.event_handler_channel.send(EventHandlerCommand {
                    cmd: EventHandlerCMD::TriggerFromEvent,
                    event: Arc::new(ev)
                });
            }

            TcpReaderCMD::WriteDataToConn => {
                // extracting data from command
                let send_data = match command.data.pop() {
                    Some(d) => d,
                    None => return
                };

                for i in 0..command.socket_token.len() {
                    match self.reader_conns.get_mut(&command.socket_token[i]) {
                        Some(conn) => {
                            // saving data to connection write queue for sending it
                            conn.add_writable_data(send_data.clone());

                            // reregistering connection as writable
                            match self.poll.reregister(&conn.socket, conn.socket_token, Ready::writable(), PollOpt::edge()) {
                                Ok(_) => {},
                                Err(e) => {
                                    warn!("Unable to reregister connection as writable for reader poll, from write data command functionality -> {}", e);
                                    continue;
                                }
                            };
                        }
                        None => continue
                    }
                }
            }

            TcpReaderCMD::WriteDataWithPath => {
                let mut ev = match command.event.pop() {
                    Some(e) => e,
                    None => return
                };

                let mut path = match BigInt::from_str(ev.path.as_str()) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Unable to convert path from event, to BigInt for handline Write By path command -> {}", e);
                        return;
                    }
                };

                let mut need_to_trigger = false;
                // if event path is dividable to current node value
                // then we need to trigger event
                if self.current_value != self.big_zero && path.clone() % self.current_value.clone() == self.big_zero {
                    path = path.clone() / self.current_value.clone();
                    need_to_trigger = true;
                }

                if path != self.big_zero {
                    self.write_by_path(path, &mut ev);
                }

                if need_to_trigger {
                    let _ = self.event_handler_channel.send(EventHandlerCommand {
                        cmd: EventHandlerCMD::TriggerFromEvent,
                        event: Arc::new(ev)
                    });
                }
            }
        }
    }

    #[inline(always)]
    fn readable(&mut self, token: Token) {
        let mut close_conn = false;
        let mut final_data: Vec<Vec<u8>> = vec![];
        {
            let mut conn = match self.reader_conns.get_mut(&token) {
                Some(c) => c,
                None => return
            };

            loop {
                if close_conn {
                    break
                }

                match conn.socket.read(&mut self.readable_buffer) {
                    Ok(rsize) => {
                        // we got EOF or not
                        if rsize == 0 {
                            close_conn = true;
                            break;
                        } else {
                            let (r_data, keep_connection) = conn.handle_data(&mut self.readable_buffer, rsize);
                            if keep_connection {
                                final_data = r_data;
                            } else {
                                close_conn = true;
                                break;
                            }
                        }

                        // if we got data less than our buffer then we got all pending data
                        // from socket buffer
                        if rsize < self.readable_buffer.len() {
                            break
                        }
                    }
                    Err(e) => {
                        // if we got WouldBlock, then this is Non Blocking socket
                        // and data still not available for this, so it's not a connection error
                        if e.kind() == ErrorKind::WouldBlock {
                            return;
                        }

                        close_conn = true;
                        break;
                    }
                }
            }
        }

        if close_conn {
            self.close_connection(token);
            return;
        }

        // if we got here then we don't need to close connection
        // so parsing data which came from socket
        loop {
            self.handle_event_data(match final_data.pop() {
                Some(d) => d,
                None => break
            })
        }
    }

    #[inline(always)]
    fn writable(&mut self, token: Token) {
        let mut close_conn = false;

        {
            let mut conn = match self.reader_conns.get_mut(&token) {
                Some(c) => c,
                None => return
            };

            match conn.flush_write_queue() {
                Ok(end_of_q) => {
                    let mut ready_state = Ready::readable();

                    // if we don't have data in our connection Queue
                    // then we need to reregister connection only for reading
                    if !end_of_q {
                        ready_state = Ready::writable();
                    }

                    match self.poll.reregister(&conn.socket, conn.socket_token, ready_state, PollOpt::edge()) {
                        Ok(_) => {},
                        Err(e) => {
                            warn!("Unable to reregister connection for reader poll, from writable functionality -> {}", e);
                            return;
                        }
                    }
                }
                Err(_) => close_conn = true
            }
        }

        if close_conn {
            self.close_connection(token);
        }
    }

    #[inline(always)]
    fn close_connection(&mut self, token: Token) {
        let conn = match self.reader_conns.remove(&token) {
            Some(c) => c,
            None => return
        };

        let mut conn_index: i32 = -1;
        // trying to find connection based on token from conn
        {
            let conns_v = match self.connections.read() {
                Ok(c) => c,
                Err(e) => {
                    warn!("Unable to set readable lock for global connections list -> {}", e);
                    return;
                }
            };

            for i in 0..conns_v.len() {
                if conns_v[i].socket_token == conn.socket_token {
                    conn_index = i as i32;
                    break;
                }
            }
        }

        // we could't find connection index
        // so we need just to return
        if conn_index < 0 {
            return;
        }

        let mut ev = Event::default();
        ev.name = String::from(EVENT_ON_CONNECTION_CLOSE);

        // if we have a connection index
        // now just locking for write and deleting specific index
        {
            let mut conns_v = match self.connections.write() {
                Ok(c) => c,
                Err(e) => {
                    warn!("Unable to set writable lock for global connections list -> {}", e);
                    return;
                }
            };

            // deleting connection here
            ev.from = conns_v.remove(conn_index as usize).token;
        }

        let _ = self.event_handler_channel.send(EventHandlerCommand {
            cmd: EventHandlerCMD::TriggerFromEvent,
            event: Arc::new(ev)
        });
    }

    #[inline(always)]
    fn handle_event_data(&mut self, buffer: Vec<u8>) {
        let mut ev = match Event::from_raw(buffer.as_ref()) {
            Ok(e) => e,
            Err(e) => {
                warn!("Error while trying to convert raw data to event object -> {}", e);
                Event{
                    path: String::new(),
                    name: String::new(),
                    from: String::new(),
                    target: String::new(),
                    public_data: String::new(),
                    data: vec![],
                }
            }
        };

        let mut path = Zero::zero();

        if !ev.path.is_empty() {
            path = match BigInt::from_str(ev.path.as_str()) {
                Ok(p) => p,
                Err(e) => {
                    warn!("Error while trying to convert parsed Event Path to BigInt path -> {}", e);
                    return;
                }
            };
        }

        // if path is 0
        // we don't need to check path combination
        if path != self.big_zero {
            self.write_by_path(path, &mut ev);
        }

        if !ev.name.is_empty() {
            let _ = self.event_handler_channel.send(EventHandlerCommand {
                cmd: EventHandlerCMD::TriggerFromEvent,
                event: Arc::new(ev)
            });
        }
    }

    fn write_by_path(&self, send_path: BigInt, ev: &mut Event) {
        let readers_len = self.reader_channels.len();
        // we will save connection token and reader index
        let mut conn_tokens: Vec<Vec<Token>> = vec![];
        for _ in 0..readers_len {
            conn_tokens.push(vec![]);
        }

        let mut path = send_path;
        // locking connections as readable for checking
        // path information from parsed event
        {
            let conns_v = match self.connections.read() {
                Ok(c) => c,
                Err(e) => {
                    warn!("Unable to set readable lock for global connections list -> {}", e);
                    return;
                }
            };

            for c in conns_v.iter() {
                if c.value == self.big_zero {
                    continue
                }

                // if connection is dividable to path then keeping divided path
                // and saving connection token for later writing to that connection
                if path.clone() % c.value.clone() == self.big_zero {
                    path = path.clone() / c.value.clone();
                    conn_tokens[c.reader_index].push(c.socket_token);
                }
            }
        }

        // setting final path
        ev.path = path.to_str_radix(10);
        let send_data = Arc::new(match ev.to_raw() {
            Ok(d) => d,
            Err(e) => {
                warn!("Unable to parse event to data with final path -> {}", e);
                return;
            }
        });

        for i in 0..readers_len {
            if conn_tokens.len() == 0 {
                break;
            }

            let tokens = conn_tokens.remove(0);
            if tokens.len() == 0 {
                continue;
            }

            // sending async command to write data for this connection
            let _ = self.reader_channels[i].send(TcpReaderCommand {
                cmd: TcpReaderCMD::WriteDataToConn,
                conn_value: vec![],
                conn: vec![],
                data: vec![send_data.clone()],
                socket_token: tokens,
                tokens: vec![],
                event: vec![]
            });
        }
    }
}
