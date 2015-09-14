package tree_db

import (
	"tree_node/node_info"
	"github.com/pquerna/ffjson/ffjson"
	"strings"
	"tree_lib"
)


func ListNodeInfos() (nfs []node_info.NodeInfo, err error) {
	err = AllKeys(DB_NODE, "", func(ns_data [][]byte){
		n_data := []byte{}
		for _, n_byte :=range ns_data {
			n := node_info.NodeInfo{}
			n_data, err = Get(DB_NODE, n_byte)
			if err != nil {
				continue
			}

			err = ffjson.Unmarshal(n_data, &n)
			if err != nil {
				continue
			}

			nfs = append(nfs, n)
		}
	})
	return
}
func ListNodeNames() (names []string, err error) {
	err = AllKeys(DB_NODE, "", func(ns_data [][]byte){
		for _, n_byte :=range ns_data {
			names = append(names, string(n_byte))
		}
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
	err = AllKeys(DB_RELATIONS, "", func(keys [][]byte)bool {
		nf := node_info.NodeInfo{}
		for _, k :=range keys {
			nf, err = GetNodeInfo(string(k))
			if err != nil {
				return false
			}
			if _, ok :=tree_lib.ArrayContains(nf.Childs, node); ok {
				parent_name = nf.Name
				return false
			}
		}

		return true
	})

	if err != nil {
		return
	}

	if len(parent_name) != 0 {
		rels = append([]string{}, parent_name, rels...)
	}

	err = Set(DB_RELATIONS, []byte(node), []byte(strings.Join(rels, ",")))
	return
}

func GetRelations(node string) ([]string, error) {
	nodes_byte, err := Get(DB_RANDOM, []byte(node))
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

	// Node relations firs element should be parent node
	return GetNodeInfo(nr[0])
}