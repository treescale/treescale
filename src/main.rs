#[macro_use]
extern crate log;
mod network;
mod node;
mod helpers;

use log::{LogLevelFilter, LogRecord, LogLevel, LogMetadata};
use node::{Node, Event, NodeConfig};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}

fn main() {
    let _ = log::set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Info);
        Box::new(SimpleLogger)
    });

    let mut node = Node::new();
    let conf = NodeConfig {
        tcp_address: String::from("0.0.0.0:8888"),
        concurrency: 2
    };

    node.on("test", Box::new(|event: &Event, _:&mut Node| -> bool {
        println!("{:?}", event.data);
        true
    }));

    node.start(conf);

    print!("{:?}", "New Implementation");
}
