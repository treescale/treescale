#![allow(dead_code)]
pub enum TcpReaderCMD {
    None
}

pub struct TcpReaderCommand {
    cmd: TcpReaderCMD
}

impl TcpReaderCommand {
    pub fn new() -> TcpReaderCommand {
        TcpReaderCommand {
            cmd: TcpReaderCMD::None
        }
    }
}