extern crate mio;

use mio::{Poll, Token, Ready, PollOpt, Events, channel};
use mio::channel::{Sender, Receiver};
use mio::tcp::{TcpListener, TcpStream};

use std::mem::size_of;
use std::sync::Mutex;

const SERVER: Token = Token(0);
const CHANNEL_READER: Token = Token(1);

struct Message {
    text: String
}

fn main() {
    println!("Token Size -> {}", size_of::<Vec<TcpStream>>());

    let addr = "0.0.0.0:8888".parse().unwrap();
    let server = TcpListener::bind(&addr).unwrap();

    let poll: Poll = Poll::new().unwrap();

    let (sender, reader): (Sender<Message>, Receiver<Message>) = channel::channel();

    poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();
    poll.register(&reader, CHANNEL_READER, Ready::readable(), PollOpt::edge()).unwrap();

    let mut events: Events = Events::with_capacity(1000);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let _ = server.accept();
                    sender.send(Message{text: String::from("Test Text from Channel")});
                }
                CHANNEL_READER => {
                    let msg: Message = reader.try_recv().unwrap();
                    println!("Text --> {}", msg.text);
                }
                _ => {
                    println!("Other Socket Event!");
                }
            }
        }
    }
}