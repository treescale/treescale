<a href="https://treescale.com" style="text-align: center; display: block">
    <img src="https://raw.githubusercontent.com/treescale/treescale/master/docs/img/tree-scale.jpg" alt="TreeScale" height="200" />
</a>

# Philosophy behind technology
Currently almost all internet connectivity working with `Request -> Response` principle which is defined by HTTP protocol as a main internet communication method. And based on growth of the internet, 
people started defining problems like <a href="https://en.wikipedia.org/wiki/C10k_problem">C10K, C100K, etc...</a> , which is all about having as match `Request -> Response` cycles as possible per second.
Now most of the softwares targeted to solve that problem is developing them specifically to handle more than 500K requests/second like an <a href="http://nginx.com">Nginx</a>, but let's think about how actually 
Linux or any other OS handling Network connection.

*On OS Networking level there is no concurrency, Network driver can read data from networking interface only once per CPU cycle, so if you have for example 5K concurrent connections, it's actually simulated by
sync operations on Networking driver. Knowing this base principle we can truly say that single connection can pass as match data as 5K (or more) concurrent connections.*

After proofing this principle TreeScale aimed to make Networking software ecosystem with always alive connections rather than using standard `Request -> Response` cycle. 
Keeping single connection is giving about
- 22% more efficient networking - because we don't have network handshake for each request
- 6% less CPU usage - because we have only one Socket opened in OS and only one stream of data from network driver
- Super reliably security layer - because we can close access for any other connections from firewall
- 100 times faster fail detection - because we have live TCP connection, and when something going down connection closing and we getting network notification within a few milliseconds

# Why we need Math Tree system
Currently almost all infrastructures have `Horizontal` scalability, which is fine for having `Request -> Response` principle with bigger time of fail detection. But because we are keeping always alive connections 
TreeScale infrastructure can't have `Master -> Slave` system like a most of the horizontal scaling systems, each node of the TreeScale infrastructure should be `Master and Slave` at the same time, and unlike Horizontal 
scalability system TreeScale should have `Truly 0 cost of Unlimited scalability`, which means if you want to scale you just plugging new server into Tree system without adding new `Master -> Slave` layer as it is in horizontal systems. 

Based on that requirements we found that Math Tree system is extremely scalable, and more reliable than horizontal scalability. 

# Software components
`TreeScale Node` is a build block os TreeScale infrastructure. Each Node is connected with live TCP connection with other nodes which is allowing to API Clients connected to Tree System, deliver data to another API client 
by just specifying Event, Group or Tag name for which that API is subscribed.

<img src="https://raw.githubusercontent.com/treescale/treescale/master/docs/img/base-structure.png" alt="TreeScale" />

**API Client** is a client API implementation for TreeScale TCP JSON protocol (Websockets also supported). Client implementation is super simple and it is completely platform independent.

**TreeScale Node** is containing main logic of infrastructure and data distribution. It is written in <a href="http://rust-lang.org" target="_blank">Rust language</a>, which is platform independent and with powerful typesystem 
we can guarantee that our software wouldn't fail!
The prototype and first version of TreeNode has been written in `Go`, then after having some integration issues and more performance requirements we rewrited it in `C++`, but after getting issues with packaging, distribution and memory management 
we decided to rewrite it into `Rust` for type safety and performance (Rust performing almost same as `C`).

As you can see on local environment network transfer rate is more than 3.4GB/s between 2 TreeNodes and 4 API clients
<img src="https://raw.githubusercontent.com/treescale/treescale/master/docs/img/bench_test.png" alt="TreeScale" height="300" />


# Application flow
1. API client connecting to one of the TreeScale Nodes (this mostly done by DNS load balancing)
2. TreeScale Node accepting connection in network layer, but it will be part of the network only when TreeScale Node would finish with authentication
3. API Client sending Authentication Token (given as an API Key) to TreeScale Node, and TreeNode is sending authentication Hook to Authentication service, that service mostly defined for specific infrastructure, so that every TreeScale software customer can have his own authentication flow.
4. After getting "success" status from Authentication service TreeNode fully accepting API Client connection, if Authentication service sending "error" status then TreeNode just closing connection with Client
5. API Client sending information about in what Group, Tag or Channel he wants to be, and list of Events for which he wants to receive events
6. TreeNode sending first event about readiness, and now Handshake process is over. API Client can send and receive events.
 
The end result of Event Publishing and Event Subscription is very similar to <a href="http://redis.io/topics/pubsub" target="_blank">Redis PubSub</a>, but unlike Redis PubSub, TreeScale allowing `0 cost unlimited scaling`, Group, Tag and channel selection 
and full data broadcasting without having Queue based system.

# Implemented Use Cases
Generally TreeScale implementing super scalable network communication, so based on that it have a lot of use-cases starting from Mobile Applications, Games and ending with Private datancenter server monitoring.
So far we have implementations for this use-cases, but we planning a lot more for the future.

- <a href="https://github.com/treescale/treescale/blob/master/docs/DockerManagement.md">Docker Orchestration and Infrastructure Management</a><br/>
- <a href="https://github.com/treescale/treescale/blob/master/docs/DockerRegistry.md">Docker Build Service, and Registry</a><br/>