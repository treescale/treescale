use network::TcpServer;
use std::sync::Arc;

pub type ServerConnectionEventCallback = Arc<dyn Fn(&Vec<u8>)>;

#[derive(PartialEq, Clone, std::cmp::Eq, std::hash::Hash)]
pub enum ServerConnectionEvents {
    OnMessage = 0,
    OnConnection = 1,
    OnDisconnect = 3,
}

pub struct Server {
    tcp_server: TcpServer,
}

impl Server {
    pub fn new(address: &str, concurrency: usize) -> Server {
        Server {
            tcp_server: TcpServer::new(address, concurrency),
        }
    }

    pub fn listen(&mut self) {
        self.tcp_server.listen()
    }

    pub fn on_message(&mut self, callback: ServerConnectionEventCallback) {
        self.tcp_server
            .on(ServerConnectionEvents::OnMessage, callback.clone())
    }
}
