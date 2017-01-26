#[macro_use]
extern crate log;
mod network;
mod event;
mod node;
mod pubsub;
use event::{Event, EVENT_ON_CONNECTION, EVENT_ON_CONNECTION_CLOSE};
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

static BIG_DATA: &'static str = "sdfgfdgdfghdfhdfghdfghgjshfdjshd sjhdg jhsdfghskdfg kjshdgkj skjdg kjs dgkjsh dkjgf skjdhg kjsh dfkgj s kjdgh kjshdfg kjsh kdjgf ksjdhfg kjdf";

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

        let mut big_str = String::new();
        for _ in 0..9000 {
            big_str.push_str(BIG_DATA);
        }

        n.on(EVENT_ON_CONNECTION, Box::new(move |ev: Arc<Event>, _: &mut Node| {
            println!("New Connection -> {}", ev.from);

            // node.emit("test_event", "25", big_str.as_str());
        }));

        n.on("test_event", Box::new(|ev: Arc<Event>, _: &mut Node| {
//             println!("Event -> {}", ev.data.len());

            // node.emit("test_event", "25", ev.data.as_str());
        }));

        n.on(EVENT_ON_CONNECTION_CLOSE, Box::new(|ev: Arc<Event>, _: &mut Node| {
            println!("Close Conn -> {}", ev.from);
        }));

        n.run();
    } else {
        let mut n = Node::new("test2", "5", "0.0.0.0:8889");
        n.on_pending_conn(Box::new(|ev: Arc<Event>, _: &mut Node| {
            println!("Got Pending Connection from -> {}", ev.from);
        }));

        n.on(EVENT_ON_CONNECTION, Box::new(|ev: Arc<Event>, _: &mut Node| {
            println!("New Connection -> {}", ev.from);
            // node.emit("test_event", "4", "test data");
        }));

        n.on("test_event", Box::new(|ev: Arc<Event>, _: &mut Node| {
             println!("{:?}", ev.data.len());
            // node.emit("test_event", "4", ev.data.as_str());
        }));

        n.on(EVENT_ON_CONNECTION_CLOSE, Box::new(|ev: Arc<Event>, _: &mut Node| {
            println!("Close Conn -> {}", ev.from);
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
