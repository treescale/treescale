package node_info
import (
	"math/big"
	"fmt"
)

// This file contains global variables for information about Tree node

type NodeInfo struct {
	Name			string			`json:"name" toml:"name" yaml:"name"`
	TreePort		int				`json:"tree_port" toml:"tree_port" yaml:"tree_port"`
	TreeIp			string			`json:"tree_ip" toml:"tree_ip" yaml:"tree_ip"`
	Tags			[]string		`json:"tags" toml:"tags" yaml:"tags"`
	Groups			[]string		`json:"groups" toml:"groups" yaml:"groups"`
	Childs			[]string		`json:"childs" toml:"childs" yaml:"childs"`
	AutoBalance		bool			`json:"auto_balance" toml:"auto_balance" yaml:"auto_balance"`
	// big.Int value for every node
	// Getting as a string from config or database and updating it to big.Int type inside node init or restart
	Value			int64			`json:"value" toml:"value" yaml:"value"`
}

var (
	// Node Info for current running node
	CurrentNodeInfo		NodeInfo
	CurrentNodeValue	*big.Int
	ParentNodeInfo		NodeInfo
	ParentNodeValue		*big.Int
	ChildsNodeInfo	=	make(map[string]NodeInfo)
	ChildsNodeValue	=	make(map[string]*big.Int)
)

func CalculateChildParentNodeValues() {
	ParentNodeValue = big.NewInt(0)
	// if SetString function will return false, then  ParentNodeValue will be big.Int with undefined value
	// We don't need to check is it ok or not
	fmt.Println(ParentNodeInfo)
	ParentNodeValue.SetInt64(ParentNodeInfo.Value)

	for n, inf :=range ChildsNodeInfo {
		ChildsNodeValue[n] = big.NewInt(0)
		// Setting value from string, but if it fails then deleting that value from MAP
		ChildsNodeValue[n].SetInt64(inf.Value)
	}
}
