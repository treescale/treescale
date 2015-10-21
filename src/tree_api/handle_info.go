package tree_api
import (
	"tree_event"
	"tree_lib"
	"tree_node/node_info"
	"tree_db"
	"tree_log"
	"github.com/pquerna/ffjson/ffjson"
	"tree_graph"
)

type Info struct {
	Target		[]string
	Group		[]string
	Tag			[]string
}

func HandleListCommand (ev *tree_event.Event, cmd Command) {
	var (
		info =  		make(map[string]node_info.NodeInfo)
		data 			[]byte
		ev_data			Command
		nodes			[]string
		nodes_in_group 	[]string
		nodes_in_tag	[]string
		err 			tree_lib.TreeError
		infos 			Info
	)
	err.From = tree_lib.FROM_HANDLE_LIST_COMMAND
	err.Err = ffjson.Unmarshal(ev.Data, &ev_data)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	err.Err = ffjson.Unmarshal(ev_data.Data, &infos)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	nodes = infos.Target
	for _, g := range infos.Group {
		nodes_in_group, err = tree_db.GetGroupNodes(g)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return
		}
		for _, n := range nodes_in_group {
			nodes = append(nodes, n)
		}
	}
	for _, t := range infos.Tag {
		nodes_in_tag, err = tree_db.GetNodesByTagName(t)
		if !err.IsNull() {
			tree_log.Error(err.From,"getting Tags", err.Error())
			return
		}
		for _, n := range nodes_in_tag {
			nodes = append(nodes, n)
		}
	}
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
	err.From = tree_lib.FROM_HANDLE_UPDATE_COMMAND
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