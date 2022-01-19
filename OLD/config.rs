#![allow(dead_code)]
extern crate clap;

use helper::Log;

use self::clap::{Arg, App};

use std::process;
use std::error::Error;

pub const APP_VERSION: &'static str = "1.0.34";
pub const MAX_API_VERSION: u32 = 1000;

pub struct NodeConfig {
    pub value: u64,
    pub token: String,
    pub api_version: u32,
    pub network: NetworkingConfig,
    pub parent_address: String
}

pub struct NetworkingConfig {
    pub tcp_server_host: String,
    pub concurrency: usize
}

pub fn parse_args() -> NodeConfig {
    let matches = App::new("TreeScale Node Service")
                    .version(APP_VERSION)
                    .author("TreeScale Inc. <hello@treescale.com>")
                    .about("TreeScale technology endpoint for event distribution and data transfer")
                    .arg(Arg::with_name("token")
                            .short("t")
                            .long("token")
                            .value_name("TOKEN")
                            .help("Token or Name for service identification, if not set, it would be auto-generated using uuid4")
                            .takes_value(true))
                    .arg(Arg::with_name("value")
                            .short("u")
                            .long("value")
                            .value_name("VALUE")
                            .help("Value for current Node, in most cases it would be generated from TreeScale Resolver")
                            .takes_value(true))
                    .arg(Arg::with_name("api")
                            .short("a")
                            .long("api")
                            .value_name("API_NUMBER")
                            .help("Sets API version for specific type of networking communications, default would be the latest version")
                            .takes_value(true))
                    .arg(Arg::with_name("parent")
                            .short("p")
                            .long("parent")
                            .value_name("PARENT_ADDRESS")
                            .takes_value(true))
                    .arg(Arg::with_name("concurrency")
                            .short("c")
                            .long("concurrency")
                            .value_name("THREADS_COUNT")
                            .help("Sets concurrency level for handling concurrent tasks, default would be cpu cores count of current machine")
                            .takes_value(true))
                    .arg(Arg::with_name("tcp_host")
                            .short("h")
                            .long("host")
                            .value_name("TCP_SERVER_HOST")
                            .help("Starts TCP server listener on give host: default is 0.0.0.0:8000")
                            .takes_value(true))
        .get_matches();

    NodeConfig {
        value: match matches.value_of("value") {
            Some(v) => match String::from(v).parse::<u64>() {
                Ok(vv) => vv,
                Err(e) => {
                    Log::error("Unable to parse given Node Value", e.description());
                    process::exit(1);
                }
            },
            None => 0
        },

        token: match matches.value_of("token") {
            Some(v) => String::from(v),
            None => String::new()
        },

        api_version: match matches.value_of("api") {
            Some(v) => match String::from(v).parse::<u32>() {
                Ok(vv) => vv,
                Err(e) => {
                    Log::error("Unable to parse given API Version", e.description());
                    process::exit(1);
                }
            },
            None => 1
        },

        network: NetworkingConfig {
            tcp_server_host: match matches.value_of("tcp_host") {
                Some(v) => String::from(v),
                None => String::from("0.0.0.0:8000")
            },
            concurrency: match matches.value_of("concurrency") {
                Some(v) => match String::from(v).parse::<usize>() {
                    Ok(vv) => vv,
                    Err(e) => {
                        Log::error("Unable to parse given Concurrency Level parameter", e.description());
                        process::exit(1);
                    }
                },
                None => 0
            },
        },

        parent_address: match matches.value_of("parent") {
            Some(v) => String::from(v),
            None => String::new()
        },
    }
}