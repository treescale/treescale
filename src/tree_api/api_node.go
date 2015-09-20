package tree_api
import (
	"tree_event"
	"tree_log"
	"tree_node/node_info"
	"tree_lib"
	"fmt"
	"tree_db"
)

const (
	API_NAME_PREFIX		=	"___TREE_API___"
	log_from_node_api	=	"Node API Backend"
)

var (
	EmitApi			func(*tree_event.Event, ...string)error
	EmitToApi		func(*tree_event.Event, ...string)error
)

func init() {
	tree_event.ON(tree_event.ON_API_CONNECTED, func(e *tree_event.Event){
		tree_log.Info(log_from_node_api, "New API client connected -> ", string(e.Data))
	})

	tree_event.ON(tree_event.ON_API_DISCONNECTED, func(e *tree_event.Event){
		tree_log.Info(log_from_node_api, "New API client disconnected -> ", string(e.Data))
	})
}

// Init API node for connection to targets
func API_INIT(targets...string) bool {
	if len(targets) == 0 {
		tree_log.Error(log_from_node_api, "For running API client you need to specify target node(s) to connect")
		return false
	}
	var err error
	for _, n :=range targets {
		node_info.ChildsNodeInfo[n], err = tree_db.GetNodeInfo(n)
		if err != nil {
			tree_log.Error(log_from_node_api, fmt.Sprintf("Unable Getting target (%s) node info from Node database, ", n), err.Error())
			return false
		}
	}

	node_info.CurrentNodeInfo = node_info.NodeInfo{
		Name: fmt.Sprintf("%s|%s", API_NAME_PREFIX, tree_lib.RandomString(10)),
		Childs: targets,
	}

	return true
}