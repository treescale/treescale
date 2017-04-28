# TreeScale: Highly scalable PubSub system

![TreeScale structure animation](https://raw.githubusercontent.com/treescale/treescale/master/animation.gif)

TreeScale is a technology which allows to build real-time PubSab applications with highly scalable architecture, using Math Tree/Graph 
 based scalability instead of standard horizontal scalability.

The goals and main philosophy behind TreeScale:
1. _Keep always alive TCP connections_ - This principle allows to avoid infinite request/response cycles, and giving more network
efficiency, faster fail detection and more secure communication.
2. _Completely Decentralized Services_ - Decentralized services are the key for infinite scalability and maximum performance, where 
one application is fully independent from another application and the base communication made by event handle/emmit.
3. _Stay Platform and Technology Independent_ - The abstraction layer over infrastructure should be independent from Application
technological stack and all kind of data transfer should be packaged as an Event for safe distribution, transfer, broadcasting.

# Building
For building from source you will need [Rust Language](https://rust-lang.org) installed. There is only one command for building this project
on all platforms which is supported by Rust.

This project mainly tested on Linux, BSD, Windows, MAC and Android(experimental).
```bash
~# git clone https://github.com/treescale/treescale
~# cd treescale

# Building with Rust package manager
~# cargo build --release

# After building see what we have now!
~# ./target/release/treescale --help
```

# Roadmap for release
- [x] Distributed Tree/Graph structure with automatic lookup
- [x] Event path calculation between Tree/Graph nodes
- [x] API Client subscriptions for each node and Event delivery
- [x] Event broadcasting and round rubin load balancing using statefull path calculation
- [ ] API Libraries for major programming languages (JavaScript, Go, Java, Python etc...)
- [ ] Queue system for each node with persistent storage (probably using RocksDB Key-Value database)
- [ ] Benchmarking with existing PubSub platforms
- [ ] Mobile integration for real time massive data delivery (already tested!)

# Contributions are welcome!
This project written in Rust because it is giving real guaranties for preventing data races and completely handles memory management
without garbage collection, which is giving huge performance improvements and low memory usage.

Project structure is simple, and everything is wrapped around single `Node` structure and associated `traits`. 

_Feel free to send pull request, open an issue even if it's not a code improvements._
