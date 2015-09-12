package tree_db

import (
	"tree_node/node_info"
	"github.com/pquerna/ffjson/ffjson"
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

func SetRelations(node string) (err error) {
	// TODO: Calculate relations for this node
	return
}