# About
Horizontal scaling is a most popular way of scaling infrastructure (or cluster), but it have a lot of disadvantages, like health-check
TTL (every check is a new TCP connection), network limitations and slow resource management. <br/>
After facing this problems, came up idea of making scaling like Mathematical Tree, which is giving advantage of having always alive 
TCP parent->childs connection (because childs count limited to 10) and make full evented future based on active connections for 
entire infrastructure.<br/>
As a starting point we are integrated Docker runtime, created Evented system as a Tree Based Networking future, for making evented 
communication between multiple servers and client side API's. <br/>
Next step will be integration with popular programming languages to give possibility, create infrastructure as a Code and write custom 
API events for specific Nodes and for entire Infrastructure Tree. 


# Core Futures
TreeScale infrastructure resource management tool have a lot of advantages compared to other projects with similar porpose but 
with Horizontal scaling, but the main advantages is
<ol>
<li>Full Docker Runtime support (it will be replaced to Tree based container system during stable releases)</li>
<li>Automatic Network 4th level load balancing for containers and child servers (based on Linux Kernel LVS)</li>
<li>Infrastructure building from config files (TOML or JSON)</li>
<li>Deep Docker container monitoring and Dashboard tools, without any additional agents or services (we are getting detailed information directly from native runtimes, without additional checking and load)</li>
</ol>

# Infrastructure Config Sample
Configuration file for automatic installation of Docker runtime and TreeScale across 100s of servers, just need's to provide 
SSH accesses and run command <code>~$ treescale build</code><br/>
<b><code>console.toml</code></b>
```toml
# Infrastructure tree config file
tree="treescale.toml"

[ssh]
    [ssh.node1]
    host="192.168.107.106"
    port="22"
    key="ssh_key.pem"
    
    [ssh.node2]
    host="192.168.107.107"
    port="22"
    username="treescale"
    password="tree"

# -------------- And so ON ------------

```
<b><code>treescale.toml</code></b>
```toml
# every node will have his local copy of this config to get all information about Tree
current_node=""

[servers]
    [servers.node1]
    name="node1"
    ip="192.168.107.106"
    service_port=8888   # port for running TreeScale Daemon
    tags=["test_node", "first_node"]    # tags for emiting events/commands to specific group of servers
    childs=["node2", "node3"]       # childs array who will be under this node in Tree structure
    [balancers]
    address="192.168.107.106:443"
    alg="rr"    # Load balancing algorithm, we are supporting 8 types of load balancing algorithms, and all of them are Network 4th level
        [containers]
        # all containers created from Docker image "ubuntu/php:dev"
        # will be load balanced to 80 port from 192.168.107.106:443
        "ubuntu/php:dev" = 80
        "ubuntu/php:test" = 5556  # balancing to 5556 port
    
    [servers.node2]
    name="node2"
    ip="192.168.107.107"
    service_port=8888   # port for running TreeScale Daemon
    tags=["test_node", "first_node"]    # tags for emiting events/commands to specific group of servers
    childs=[]       # childs array who will be under this node in Tree structure    

```

# Stay in touch
Feel free to contact <a href="mailto:tigran@treescale.com">tigran@treescale.com</a>. <br/>
We would love to answer your questions and resolve your Github issues :)