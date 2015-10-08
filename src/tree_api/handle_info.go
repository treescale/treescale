package tree_api
import (
	"tree_event"
	"tree_lib"
	"strings"
	"tree_node/node_info"
	"tree_db"
	"tree_log"
	"github.com/pquerna/ffjson/ffjson"
	"fmt"
	"tree_graph"
)


func HandleListCommand (ev *tree_event.Event, cmd Command) {
	var (
		info =  		make(map[string]node_info.NodeInfo)
		data 			[]byte
		ev_data			Command
		nodes			[]string
		err 			tree_lib.TreeError
	)
	fmt.Println(string(ev.Data))
	err.Err = ffjson.Unmarshal(ev.Data, &ev_data)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	nodes = strings.Split(string(ev_data.Data), ",")
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
	var (
		data 			Command
		info 			node_info.NodeInfo
		err 			tree_lib.TreeError
	)
	err.From = "From HandleUpdateCommand"
	err.Err = ffjson.Unmarshal(ev.Data, &data)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
	}
	err.Err = ffjson.Unmarshal(data.Data, &info)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
	}
	UpdateNodeChange(info)
	SendCommandCallback(ev,ev.Data)
}

func UpdateNodeChange (info node_info.NodeInfo) {
	var (
		ev 			=	&tree_event.Event{}
		err 			tree_lib.TreeError
	)
	err.From = tree_lib.FROM_UPDATE_NODE_CHANGE
	fmt.Println(info, node_info.CurrentNodeInfo.Name)
	ev.Data, err.Err = ffjson.Marshal(info)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	path := &tree_graph.Path{From: node_info.CurrentNodeInfo.Name, Nodes: []string{"*"} }
	ev.Name = tree_event.ON_UPDATE_NODE_INFO
	tree_event.Trigger(ev)
	tree_event.Emit(ev, path)
}