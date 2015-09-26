package tree_db

import (
	"tree_node/node_info"
	"github.com/pquerna/ffjson/ffjson"
	"strings"
	"tree_lib"
	"github.com/syndtr/goleveldb/leveldb/errors"
	"strconv"
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
		nodes 				[]string
		prime 				int64
	)
	err.From = tree_lib.FROM_SET_PATH_VALUES
	nodes, err = ListNodeNames()
	if !err.IsNull() {
		return
	}
	prime = 2
	for _, n := range nodes {
		err = Set(DB_PATH_VALUE,[]byte(n),[]byte(string(prime)))
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
	)
	err.From = tree_lib.FROM_GET_NODE_VALUE
	data, err = Get(DB_PATH_VALUE, []byte(node))
	if !err.IsNull() {
		return
	}
	value, err.Err = strconv.ParseInt(string(data), 10, 64)
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

	return strings.Split(string(nodes_byte), ","), err
}

func GroupAddNode(group, node string) (err tree_lib.TreeError) {
	var gB []byte
	err.From = tree_lib.FROM_GROUP_ADD_NODE
	gB, err = Get(DB_GROUP, []byte(group))
	if !err.IsNull() {
		return
	}

	group_nodes := strings.Split(string(gB), ",")
	group_nodes = append(group_nodes, node)

	err = Set(DB_GROUP, []byte(group), []byte(strings.Join(group_nodes, ",")))
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
	var gB []byte
	err.From = tree_lib.FROM_TAG_ADD_NODE
	gB, err = Get(DB_TAG, []byte(tag))
	if !err.IsNull() {
		return
	}

	tag_nodes := strings.Split(string(gB), ",")
	tag_nodes = append(tag_nodes, node)

	err = Set(DB_TAG, []byte(tag), []byte(strings.Join(tag_nodes, ",")))
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
	nr, err := GetRelations(node)
	err.From = tree_lib.FROM_GET_PARENT_INFO
	if !err.IsNull() {
		return node_info.NodeInfo{}, err
	}
	if len(nr[0]) == 0 {
		return node_info.NodeInfo{}, err
	}

	// Node relations firs element should be parent node
	return GetNodeInfo(nr[0])
}