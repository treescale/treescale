package tree_api
import (
	event "tree_event"
	"tree_log"
)

const (
	API_NAME_PREFIX		=	"___TREE_API___"
	log_from_node_api	=	"Node API Backend"
)

func init() {
	event.ON(event.ON_API_CONNECTED, func(e *event.Event){
		tree_log.Info(log_from_node_api, "New API client connected -> ", string(e.Data))
	})

	event.ON(event.ON_API_DISCONNECTED, func(e *event.Event){
		tree_log.Info(log_from_node_api, "New API client disconnected -> ", string(e.Data))
	})
}