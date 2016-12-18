#[macro_use]
extern crate log;
mod network;
mod event;
mod node;
use event::{Event, EVENT_ON_CONNECTION};
use node::Node;
use std::sync::Arc;
use std::env;

use log::{LogLevelFilter, LogRecord, LogLevel, LogMetadata};

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

    let args: Vec<String> = env::args().collect();
    if args[1] == String::from("t1") {
        let mut n = Node::new("test1", "2", "0.0.0.0:8888");
        n.on_pending_conn(Box::new(|ev: Arc<Event>, node: &mut Node| {
            println!("Got Pending Connection from -> {}", ev.from);
            node.accept(ev.from.clone());
        }));

        n.on(EVENT_ON_CONNECTION, Box::new(|ev: Arc<Event>, _: &mut Node| {
            println!("New Connection -> {}", ev.from);
        }));

        n.run();
    } else {
        let mut n = Node::new("test2", "5", "0.0.0.0:8889");
        n.on_pending_conn(Box::new(|ev: Arc<Event>, _: &mut Node| {
            println!("Got Pending Connection from -> {}", ev.from);
        }));

        n.on(EVENT_ON_CONNECTION, Box::new(|ev: Arc<Event>, _: &mut Node| {
            println!("New Connection -> {}", ev.from);
        }));

        match n.connect("127.0.0.1:8888") {
            Ok(_) => {},
            Err(e) => {
                println!("Unable to connect !! -> {}" , e);
                return;
            }
        }
        n.run();
    }

    println!("{}", "sdff");
}
