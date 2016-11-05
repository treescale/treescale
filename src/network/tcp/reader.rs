extern crate mio;

use self::mio::{Token, Event, Poll, Ready, PollOpt, Evented, Events};
use self::mio::channel::{Sender, Receiver, channel};
use network::tcp::connection::ReaderConnection;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub type MutexQueue<A, B> = Mutex<HashMap<A, Vec<Vec<B>>>>;

const CHANNEL_TOKEN: Token = Token(1);

enum ReaderCommandsTypes {
    StopEventLoop,
}

pub struct ReaderCommand {
    cmd: ReaderCommandsTypes,
}

pub struct Reader {
    // Queue of writable data, using specific connection token and data Vector
    // this will be set from outside of the loop, that's why it is set as a mutex
    write_queue: Arc<MutexQueue<Token, u8>>,
    connections: HashMap<Token, ReaderConnection>,
    sender_channel: Sender<fn(&Reader)>,
    receiver_channel: Receiver<fn(&Reader)>
}

impl Reader {
    pub fn run(&mut self) {
        let poll: Poll = match Poll::new() {
            Ok(p) => p,
            Err(e) => {
                println!("Unable to create Event Loop for Reader -> {:}", e);
                return;
            }
        };

        let (sender, reader): (Sender<fn(&Reader)>, Receiver<fn(&Reader)>) = channel();
        self.sender_channel = sender;
        poll.register(&reader, CHANNEL_TOKEN, Ready::readable(), PollOpt::edge());

        let mut events: Events = Events::with_capacity(1000);

        loop {
            match poll.poll(&mut events, None) {
                Ok(event_count) => {
                    if event_count == 0 {
                        continue;
                    }

                    for event in events.iter() {
                        let event_token = event.token();
                        if event_token == CHANNEL_TOKEN {
                            match reader.try_recv() {
                                Ok(callback) => {
                                    // callback for command implementation
                                    callback(&self);
                                }
                                Err(_) => {}
                            }

                            continue;
                        }

                        // if we don't have connection token, just moving forward
                        if !self.connections.contains_key(&event_token) {
                            continue;
                        }

                        let event_kind = event.kind();

                        if event_kind.is_readable() {
                            // TODO: Read from connection
                            // self.connections[&event_token]
                        } else if event_kind.is_writable() {
                            // TODO: Write to connection
                            // self.connections[&event_token]
                        } else if event_kind.is_hup() {
                            // TODO: handle HUP type
                        } else if event_kind.is_error() {
                            // TODO: handle ERROR for socket
                        }
                    }
                }
                Err(e) => {
                    // TODO: Handle error here
                    return;
                }
            }
        }
    }

    fn channel(&self) -> Sender<fn(&Reader)> {
        return self.sender_channel.clone();
    }
}
