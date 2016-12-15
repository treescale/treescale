extern crate mio;
extern crate num;

use network::tcp::{TcpConnValue, TcpConn};
use std::sync::{Arc, RwLock};
use self::mio::channel::{Sender, Receiver, channel};
use self::mio::{Token, Poll};
use self::num::{BigInt, Zero};
use std::process;

pub enum TcpReaderCMD {
    HandleConnection
}

pub struct TcpReaderCommand {
    pub cmd: TcpReaderCMD,
    pub conn_value: Vec<TcpConnValue>,
    pub conn: Vec<TcpConn>
}

pub struct TcpReader {
    connections: Arc<RwLock<Vec<TcpConnValue>>>,

    // reader sender channels
    sender_channel: Sender<TcpReaderCommand>,
    receiver_channel: Receiver<TcpReaderCommand>,

    // Vector of channels for readers including current one
    pub reader_channels: Vec<Sender<TcpReaderCommand>>,
    pub reader_index: usize,

    poll: Poll,
    big_zero: BigInt
}

impl TcpReader {
    pub fn new(connections: Arc<RwLock<Vec<TcpConnValue>>>) -> TcpReader {
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
            big_zero: Zero::zero()
        }
    }

    pub fn channel(&self) -> Sender<TcpReaderCommand> {
        self.sender_channel.clone()
    }

    pub fn run(&mut self) {

    }

    #[inline(always)]
    fn notify(&mut self, command: &mut TcpReaderCommand) {

    }

    #[inline(always)]
    fn readable(&mut self, token: Token) {

    }

    #[inline(always)]
    fn writeabale(&mut self, token: Token) {

    }

    #[inline(always)]
    fn close_connection(&mut self, token: Token) {

    }
}
