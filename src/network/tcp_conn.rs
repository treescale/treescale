extern crate mio;
extern crate num;

use mio::{Token};
use mio::tcp::TcpStream;
use self::num::bigint::{BigInt, Sign};

pub struct TcpConnection {
    from_server: bool,
    // this will be token for handling MIO connection loop
    key: Token,
    token: String,
    value: BigInt,
    sock: TcpStream,
}

impl TcpConnection {
    pub fn new(sock: TcpStream, token: Token, from_server: bool) -> TcpConnection {
        TcpConnection {
            sock: sock,
            key: token,
            token: String::new(),
            value: BigInt::new(Sign::Plus, vec![0]),
            from_server: from_server
        }
    }
}
