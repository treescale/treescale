package tree_db

import (
	"tree_node/node_info"
	"github.com/pquerna/ffjson/ffjson"
	"strings"
	"tree_lib"
	"github.com/syndtr/goleveldb/leveldb/errors"
)


func ListNodeInfos() (nfs []node_info.NodeInfo, err error) {
	err = ForEach(DB_NODE, func(key []byte, val []byte)error {
		n := node_info.NodeInfo{}
		err = ffjson.Unmarshal(val, &n)
		if err != nil {
			return err
		}
		nfs = append(nfs, n)
		return nil
	})
	return
}
func ListNodeNames() (names []string, err error) {
	err = ForEach(DB_NODE, func(key []byte, value []byte)error {
		names = append(names, string(key))
		return nil
	})
	return
}

func GetNodeInfo(name string) (nf node_info.NodeInfo, err error) {
	var (
		value	[]byte
	)

	value, err = Get(DB_NODE, []byte(name))
	if err != nil {
		return
	}

	err = ffjson.Unmarshal(value, &nf)
	return
}

func SetNodeInfo(name string, nf node_info.NodeInfo) (err error) {
	var (
		value	[]byte
	)

	value, err = ffjson.Marshal(nf)
	if err != nil {
		return
	}

	err = Set(DB_NODE, []byte(name), value)
	return
}

// Key -> value ..... node_name -> node1,node2,node3
// []byte -> []string{}.Join(",")
// First element of string array should be parent node
func SetRelations(node string) (err error) {
	parent_name := ""
	inf := node_info.NodeInfo{}
	inf, err = GetNodeInfo(node)
	if err != nil {
		return
	}

	rels := inf.Childs

	// Getting parent node
	err = ForEach(DB_NODE, func(key []byte, val []byte)error {
		nf := node_info.NodeInfo{}
		err = ffjson.Unmarshal(val, &nf)
		if err != nil {
			return err
		}
		if _, ok :=tree_lib.ArrayContains(nf.Childs, node); ok {
			parent_name = nf.Name
			return errors.New("")  // Just ending the ForEach with empty error
		}

		return nil
	})

	if err != nil {
		return
	}

	if len(parent_name) != 0 {
		rels = append(append([]string{}, parent_name), rels...)
	}

	err = Set(DB_RELATIONS, []byte(node), []byte(strings.Join(rels, ",")))
	return
}

func GetRelations(node string) ([]string, error) {
	nodes_byte, err := Get(DB_RELATIONS, []byte(node))
	if err != nil {
		return nil, err
	}

	return strings.Split(string(nodes_byte), ","), nil
}

func GetGroupNodes(group string) ([]string, error) {
	nodes_byte, err := Get(DB_GROUP, []byte(group))
	if err != nil {
		return nil, err
	}

	return strings.Split(string(nodes_byte), ","), nil
}
func GetNodesByTagName(tag string) ([]string, error) {
	nodes_byte, err := Get(DB_TAG, []byte(tag))
	if err != nil {
		return nil, err
	}
	return strings.Split(string(nodes_byte), ","), nil
}

func GetParentInfo(node string) (node_info.NodeInfo, error) {
	nr, err := GetRelations(node)
	if err != nil {
		return node_info.NodeInfo{}, err
	}
	if len(nr[0]) == 0 {
		return node_info.NodeInfo{}, nil
	}

	// Node relations firs element should be parent node
	return GetNodeInfo(nr[0])
}