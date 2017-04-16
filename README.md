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
<li>Evented infrastructure, with custom events and handlers</li>
<li>Full Docker Runtime support (it will be replaced to Tree based container system during stable releases)</li>
<li>Automatic Network 4th level load balancing for containers and child servers (based on Linux Kernel LVS)</li>
<li>Infrastructure building from config files (TOML, JSON or YAML)</li>
<li>Deep Docker container monitoring and Dashboard tools, without any additional agents or services (we are getting detailed information directly from native runtimes, without additional checking and load)</li>
</ol>

# Infrastructure Config Sample
Configuration file for automatic installation of Docker runtime and TreeScale across 100s of servers, just need's to provide 
SSH accesses and run command <code>~$ treescale build</code><br/>
<b><code>ssh.toml</code></b>
```toml
# Infrastructure servers SSH config
[ssh]
    # using SSH Agent 
    [ssh.tree1]
    host="192.168.107.106"
    port="22"
    username="vagrant"
    
    # Using username and password
    [ssh.tree2]
    host="192.168.107.107"
    port="22"
    username="treescale"
    password="tree"

# -------------- And so ON ------------

```
<b><code>tree.toml</code></b>

```toml
# every node will have his local copy of this config inside local small database file for getting all information about Tree

[tree_node]
    [tree_node.tree1]
    name="tree1"
    tree_port=8888 # port for running TreeScale Daemon
    tree_ip="tree-node-1.cloudapp.net"
    tags=[] # tags for emiting events/commands to specific group of servers
    groups=["backend"] # groups array who will be under this node in Tree structure
    childs=["tree2", "tree4"] # childs array who will be under this node in Tree structure
    
    [tree_node.tree2]
    name="tree2"
    tree_port=8888
    tree_ip="tree-node-2.cloudapp.net"
    tags=[]
    groups=[]
    childs=["tree3"]
    
    [tree_node.tree3]
    name="tree3"
    tree_port=8888
    tree_ip="tree-node-3.cloudapp.net"
    tags=[]
    groups=[]
    childs=[]
    
    [tree_node.tree4]
    name="tree4"
    tree_port=8888
    tree_ip="tree-node-4.cloudapp.net"
    tags=[]
    groups=[]
    childs=["tree6", "tree7"]
```

# Get Started
Our System using <a href="https://github.com/boltdb/bolt">BoltDB</a> as a local storage engine, so you need to provide path to local db file <code>TREE_DB_PATH</code>
environment variable example: <code>export TREE_DB_PATH="./tree.db"</code> . Default Path is <code>/etc/treescale/tree.db</code>
<p>
For Getting started with our config files which is available in our repository <code>configs/tree.toml, configs/ssh.toml</code> you need to do following commands
</p>

```bash
# Before using TreeScale you need to have Tree Based infrastructure, 
# which could be built using configuration files with SSH access and Tree nodes information
treescale build  --files=configs/tree.toml,configs/ssh.toml

# Compiling config files into BoltDB storage for faster work, output file will be tree.db
treescale config compile -p ./configs -o tree.db

# Next we need to restore database to our given TREE_DB_PATH
treescale config restore -f tree.db

# After this step you can now use TreeScale CLI commands to manipulate your infrastructure
 
# this command will execute bash command on tree3 server by connecting to tree1 server as an API server
# it's a base Tree structure path calculation
treescale api exec -c "uname" -n tree1 -t tree3
```

# Wiki Pages
We are adding documentation to our Wiki pages and it will be ready very soon.<br/>
But before that you can check our helpers just to understand our command stack and use cases.
<code>treescale [command] --help</code>

# Stay in touch
Feel free to contact <a href="mailto:tigran@treescale.com">tigran@treescale.com</a>. <br/>
We would love to answer your questions and resolve your Github issues :)
