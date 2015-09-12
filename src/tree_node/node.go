package tree_node
import (
	"tree_db"
	"tree_node/node_info"
	"tree_log"
	"fmt"
	"tree_net"
)


var (
	log_from_node		=	"Node functionality"
	current_node_name		string
)

func SetParent(name string) bool {
	var err error
	node_info.ParentNodeInfo, err = tree_db.GetNodeInfo(name)
	if err != nil {
		tree_log.Error(log_from_node, "Getting parent node info from Node database, ", err.Error())
		return false
	}
	return true
}

func node_init() {
	// Getting current node name
	current_node_byte, err := tree_db.Get(tree_db.DB_RANDOM, []byte("current_node"))
	if err != nil {
		tree_log.Error(log_from_node, "Getting current node name from Random database, ", err.Error())
		return
	}
	current_node_name = string(current_node_byte)
	node_info.CurrentNodeInfo, err = tree_db.GetNodeInfo(current_node_name)
	if err != nil {
		tree_log.Error(log_from_node, "Getting current node info from Node database, ", err.Error())
		return
	}
	for _, child :=range node_info.ChildsNodeInfo {
		node_info.ChildsNodeInfo[child], err = tree_db.GetNodeInfo(child)
		if err != nil {
			tree_log.Error(log_from_node, fmt.Sprintf("Getting child (%s) node info from Node database, ", child), err.Error())
			return
		}
	}

	// Setting relations
	tree_db.SetRelations(current_node_name)

	node_info.ParentNodeInfo == tree_db.GetParentInfo(current_node_name)
}

func Start() {
	node_init()
	tree_net.Start()
	return
}

func Restart() {
	node_init()
	tree_net.Restart()
}