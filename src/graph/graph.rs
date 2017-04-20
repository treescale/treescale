use std::collections::BTreeMap;

use graph::Subscriptions;

/// TreeNode keeping information about individual nodes information
/// Including subscribed events, channels, groups list
pub struct TreeNode {
    // Token for this Node
    token: String,
    // Prime value for this Node
    value: u64,
    // List of node tokens who are connected to this Node
    relations: Vec<String>
}

/// Graph struct is the main structure for keeping tree state information
/// And keeping events, channels, group names for tree
pub struct Graph {
    // Map of Nodes inside this Graph
    // Key -> Node Token
    // Value -> TreeNode
    nodes: BTreeMap<String, TreeNode>,

    // Events Map inside this Graph
    // Key -> Event Name
    // Value -> Node Token
    events: Subscriptions,

    // Channels Map inside this Graph
    // Key -> Channel Name
    // Value -> Node Token
    channels: Subscriptions,

    // Groups Map inside this Graph
    // Key -> Group Name
    // Value -> Channel Name
    groups: Subscriptions,
}