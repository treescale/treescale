#![allow(dead_code)]
pub enum TcpWriterCMD {
    None
}

pub struct TcpWriterCommand {
    cmd: TcpWriterCMD
}

impl TcpWriterCommand {
    pub fn new() -> TcpWriterCommand {
        TcpWriterCommand {
            cmd: TcpWriterCMD::None
        }
    }
}