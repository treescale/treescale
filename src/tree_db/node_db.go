package tree_db

import (
	"tree_node/node_info"
	"github.com/pquerna/ffjson/ffjson"
	"strings"
	"tree_lib"
	"github.com/syndtr/goleveldb/leveldb/errors"
	"fmt"
)


func ListNodeInfos() (nfs []node_info.NodeInfo, err tree_lib.TreeError) {
	err.From = tree_lib.FROM_LIST_NODE_INFOS
	err = ForEach(DB_NODE, func(key []byte, val []byte)error {
		n := node_info.NodeInfo{}
		err := ffjson.Unmarshal(val, &n)
		if err != nil {
			return err
		}
		nfs = append(nfs, n)
		return nil
	})
	return
}
func ListNodeNames() (names []string, err tree_lib.TreeError) {
	err.From = tree_lib.FROM_LIST_NODE_NAMES
	err = ForEach(DB_NODE, func(key []byte, value []byte)error {
		names = append(names, string(key))
		return nil
	})
	return
}

func GetNodeInfo(name string) (nf node_info.NodeInfo, err tree_lib.TreeError) {
	var (
		value	[]byte
	)
	err.From = tree_lib.FROM_GET_NODE_INFO
	value, err = Get(DB_NODE, []byte(name))
	if !err.IsNull() {
		return
	}

	err.Err = ffjson.Unmarshal(value, &nf)
	return
}

func SetNodeInfo(name string, nf node_info.NodeInfo) (err tree_lib.TreeError) {
	var (
		value	[]byte
	)
	err.From = tree_lib.FROM_SET_NODE_INFO
	value, err.Err = ffjson.Marshal(nf)
	if !err.IsNull() {
		return
	}

	err = Set(DB_NODE, []byte(name), value)
	return
}

// Key -> value ..... node_name -> node1,node2,node3
// []byte -> []string{}.Join(",")
// First element of string array should be parent node
func SetRelations(node string) (err tree_lib.TreeError) {
	err.From = tree_lib.FROM_SET_RELATIONS
	parent_name := ""
	inf := node_info.NodeInfo{}
	inf, err = GetNodeInfo(node)
	if !err.IsNull() {
		return
	}

	rels := inf.Childs

	// Getting parent node
	err = ForEach(DB_NODE, func(key []byte, val []byte)error {
		nf := node_info.NodeInfo{}
		err := ffjson.Unmarshal(val, &nf)
		if err != nil {
			return err
		}
		if _, ok :=tree_lib.ArrayContains(nf.Childs, node); ok {
			parent_name = nf.Name
			return errors.New("")  // Just ending the ForEach with empty error
		}

		return nil
	})

	if !err.IsNull() {
		return
	}

	if len(parent_name) != 0 {
		rels = append(append([]string{}, parent_name), rels...)
	}

	err = Set(DB_RELATIONS, []byte(node), []byte(strings.Join(rels, ",")))
	return
}
func SetPathValues() (err tree_lib.TreeError) {
	var (
		nodes 				[]node_info.NodeInfo
		prime 				int64
		value				[]byte
	)
	err.From = tree_lib.FROM_SET_PATH_VALUES
	nodes, err = ListNodeInfos()
	if !err.IsNull() {
		return
	}
	prime = 2
	for _, n := range nodes {
		n.Value = prime
		value, err.Err = ffjson.Marshal(n)
		if !err.IsNull() {
			return
		}
		err = Set(DB_NODE,[]byte(n.Name),value)
		if !err.IsNull(){
			return
		}
		prime = tree_lib.NextPrimeNumber(prime)
	}
	return
}
func GetNodeValue(node string) (value int64, err tree_lib.TreeError) {
	var (
		data 		[]byte
		info		node_info.NodeInfo
	)
	err.From = tree_lib.FROM_GET_NODE_VALUE
	data, err = Get(DB_NODE, []byte(node))
	if !err.IsNull() {
		return
	}
	err.Err = ffjson.Unmarshal(data, &info)
	value = info.Value
	return
}
func GetRelations(node string) ([]string, tree_lib.TreeError) {
	nodes_byte, err := Get(DB_RELATIONS, []byte(node))
	err.From = tree_lib.FROM_GET_RELATIONS
	if !err.IsNull() {
		return nil, err
	}

	return strings.Split(string(nodes_byte), ","), err
}

func GetGroupNodes(group string) ([]string, tree_lib.TreeError) {
	nodes_byte, err := Get(DB_GROUP, []byte(group))
	err.From = tree_lib.FROM_GET_GROUP_NODES
	if !err.IsNull() {
		return nil, err
	}
	if nodes_byte != nil {
		return strings.Split(string(nodes_byte), ","), err
	}
	return []string{}, err
}

func GroupAddNode(group, node string) (err tree_lib.TreeError) {
	var (
		gB 				[]byte
		group_nodes		[]string
	)
	err.From = tree_lib.FROM_GROUP_ADD_NODE
	gB, err = Get(DB_GROUP, []byte(group))
	if !err.IsNull() {
		return
	}
	if len(gB) > 0 {
		group_nodes = strings.Split(string(gB), ",")
	}
	if _, ok := tree_lib.ArrayContains(group_nodes, node); !ok {
		group_nodes = append(group_nodes, node)
	}

	err = Set(DB_GROUP, []byte(group), []byte(strings.Join(group_nodes, ",")))
	return
}
func GroupDeleteNode(group, node string) (err tree_lib.TreeError) {
	var (
		gB 				[]byte
		group_nodes		[]string
	)
	err.From = tree_lib.FROM_GROUP_ADD_NODE
	gB, err = Get(DB_GROUP, []byte(group))
	if !err.IsNull() {
		return
	}
	if len(gB) > 0 {
		group_nodes = strings.Split(string(gB), ",")
	}
	if n, ok := tree_lib.ArrayContains(group_nodes, node); ok {
		group_nodes = group_nodes[:n+copy(group_nodes[n:], group_nodes[n+1:])]
	}
	fmt.Println(group_nodes)
	err = Set(DB_GROUP, []byte(group), []byte(strings.Join(group_nodes, ",")))
	return
}
func DeleteNodeFromHisGroups(node string) (err tree_lib.TreeError){
	var nf node_info.NodeInfo
	err.From = tree_lib.FROM_ADD_NODE_TO_HIS_GROUPS
	nf, err = GetNodeInfo(node)
	if !err.IsNull() {
		return
	}

	for _, g :=range nf.Groups {
		err = GroupDeleteNode(g, node)
		if !err.IsNull() {
			return
		}
	}
	return
}
func AddNodeToHisGroups(node string) (err tree_lib.TreeError) {
	var nf node_info.NodeInfo
	err.From = tree_lib.FROM_ADD_NODE_TO_HIS_GROUPS
	nf, err = GetNodeInfo(node)
	if !err.IsNull() {
		return
	}

	for _, g :=range nf.Groups {
		err = GroupAddNode(g, node)
		if !err.IsNull() {
			return
		}
	}
	return
}

func GetNodesByTagName(tag string) ([]string, tree_lib.TreeError) {
	nodes_byte, err := Get(DB_TAG, []byte(tag))
	err.From = tree_lib.FROM_GET_NODES_BY_TAG_NAME
	if !err.IsNull() {
		return nil, err
	}
	return strings.Split(string(nodes_byte), ","), err
}

func TagAddNode(tag, node string) (err tree_lib.TreeError) {
	var (
		gB 			[]byte
		tag_nodes	[]string
	)
	err.From = tree_lib.FROM_TAG_ADD_NODE
	gB, err = Get(DB_TAG, []byte(tag))
	if !err.IsNull() {
		return
	}
	if len(gB) > 0 {
		tag_nodes = strings.Split(string(gB), ",")
	}
	if _, ok := tree_lib.ArrayContains(tag_nodes, node); !ok {
		tag_nodes = append(tag_nodes, node)
	}
	err = Set(DB_TAG, []byte(tag), []byte(strings.Join(tag_nodes, ",")))
	return
}
func TagDeleteNode (tag, node string) (err tree_lib.TreeError) {
	var (
		gB 			[]byte
		tag_nodes	[]string
	)
	err.From = tree_lib.FROM_TAG_ADD_NODE
	gB, err = Get(DB_TAG, []byte(tag))
	if !err.IsNull() {
		return
	}
	if len(gB) > 0 {
		tag_nodes = strings.Split(string(gB), ",")
	}
	if n, ok := tree_lib.ArrayContains(tag_nodes, node); !ok {
		tag_nodes = tag_nodes[:n+copy(tag_nodes[n:], tag_nodes[n+1:])]
	}
	err = Set(DB_TAG, []byte(tag), []byte(strings.Join(tag_nodes, ",")))
	return
}
func DeleteNodeFromHisTags(node string) (err tree_lib.TreeError) {
	var nf node_info.NodeInfo
	err.From = tree_lib.FROM_ADD_NODE_TO_HIS_TAGS
	nf, err = GetNodeInfo(node)
	if !err.IsNull() {
		return
	}

	for _, t :=range nf.Tags {
		err = TagDeleteNode(t, node)
		if !err.IsNull() {
			return
		}
	}
	return
}
func AddNodeToHisTags(node string) (err tree_lib.TreeError) {
	var nf node_info.NodeInfo
	err.From = tree_lib.FROM_ADD_NODE_TO_HIS_TAGS
	nf, err = GetNodeInfo(node)
	if !err.IsNull() {
		return
	}

	for _, t :=range nf.Tags {
		err = TagAddNode(t, node)
		if !err.IsNull() {
			return
		}
	}
	return
}


func GetParentInfo(node string) (node_info.NodeInfo, tree_lib.TreeError) {
	var (
		err 	tree_lib.TreeError
		pname	string
	)
	err.From = tree_lib.FROM_GET_PARENT_INFO

	err = ForEach(DB_NODE, func(key []byte, val []byte)error {
		n := node_info.NodeInfo{}
		err := ffjson.Unmarshal(val, &n)
		if err != nil {
			return err
		}

		if _, ok := tree_lib.ArrayContains(n.Childs, node); ok {
			pname = n.Name
			return errors.New("Just Err for break")
		}

		return nil
	})

	if len(pname) == 0 {
		return node_info.NodeInfo{}, tree_lib.TreeError{}
	}

	// Node relations first element should be parent node
	return GetNodeInfo(pname)
}