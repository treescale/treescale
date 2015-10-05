package tree_api
import (
	"tree_event"
	"tree_lib"
	"strings"
	"tree_node/node_info"
	"tree_db"
	"tree_log"
	"github.com/pquerna/ffjson/ffjson"
)


func HandleListCommand (ev *tree_event.Event, cmd Command) {
	var (
		info =  		make(map[string]node_info.NodeInfo)
		data 			[]byte
		nodes			[]string
		err 			tree_lib.TreeError
	)
	nodes = strings.Split(string(ev.Data), ",")
	for _, n := range nodes {
		info[n], err = tree_db.GetNodeInfo(n)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return
		}
	}
	cb_cmd := cmd
	cb_cmd.Data, err.Err = ffjson.Marshal(info)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	data, err.Err = ffjson.Marshal(cb_cmd)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	SendCommandCallback(ev, data)
}

func HandleUpdateCommand(ev *tree_event.Event, cmd Command){
//	var (
//		info 			node_info.NodeInfo
//		err 			tree_lib.TreeError
//	)
//	err.Err = ffjson.Unmarshal(ev.Data, info)
//	if !err.IsNull() {
//		tree_log.Error(err.From, err.Error())
//	}
//	UpdateNodeChange(info)
}

//func UpdateNodeChange (info node_info.NodeInfo) {
//	var (
//		ev 				*tree_event.Event
//		emitter 		*tree_event.EventEmitter
//		err 			tree_lib.TreeError
//		path			*big.Int
//	)
//	err.From = tree_lib.FROM_UPDATE_NODE_CHANGE
//	ev.Data, err.Err = ffjson.Marshal(info)
//	if !err.IsNull() {
//		tree_log.Error(err.From, err.Error())
//		return
//	}
//	path, err = tree_graph.GetPath(node_info.CurrentNodeInfo.Name, []string{"*"},[]string{},[]string{})
//	ev.Name = tree_event.ON_UPDATE_NODE_INFO
//	tree_event.Trigger(ev)
//	emitter.Data = ev.Data
//	emitter.Name = ev.Name
//	emitter.Path = path
//	tree_event.Emit(emitter)
//}