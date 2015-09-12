package node_info

// This file contains global variables for information about Tree node

type NodeInfo struct {
	Name			string			`json:"name" toml:"name" yaml:"name"`
	TreePort		int				`json:"tree_port" toml:"tree_port" yaml:"tree_port"`
	TreeIp			string			`json:"tree_ip" toml:"tree_ip" yaml:"tree_ip"`
	Tags			[]string		`json:"tags" toml:"tags" yaml:"tags"`
	Groups			[]string		`json:"groups" toml:"groups" yaml:"groups"`
	Childs			[]string		`json:"childs" toml:"childs" yaml:"childs"`
}

var (
	// Node Info for current running node
	CurrentNodeInfo		*NodeInfo
	ParentNodeInfo		*NodeInfo
	ChildsNodeInfo	=	make(map[string]*NodeInfo)
)
