package tree_db

import (
	"tree_node/node_info"
	"errors"
	"github.com/pquerna/ffjson/ffjson"
)


func ListNodes() (nfs []node_info.NodeInfo, err error) {
	if node_db == nil {
		err = errors.New("Node database is not selected, please select before making query")
		return
	}


	return
}

func GetNodeInfo(name string) (nf node_info.NodeInfo, err error) {
	var (
		value	[]byte
	)
	if node_db == nil {
		err = errors.New("Node database is not selected, please select before making query")
		return
	}

	value, err = node_db.Get([]byte(name))
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
	if node_db == nil {
		err = errors.New("Node database is not selected, please select before making query")
		return
	}
	value, err = ffjson.Marshal(nf)
	if err != nil {
		return
	}

	err = node_db.Set([]byte(name), value)
	return
}