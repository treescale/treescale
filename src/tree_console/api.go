package tree_console

import (
	"github.com/spf13/cobra"
	"tree_log"
	"fmt"
	"tree_api"
	"tree_event"
	"tree_lib"
	"tree_graph"
	"tree_node/node_info"
	"github.com/pquerna/ffjson/ffjson"
	"tree_graph/path"
	"strings"
)


func HandleApiExec(cmd *cobra.Command, args []string) {
	var (
		nodes 			[]string
		targets 		[]string
		target_groups 	[]string
		target_tags 	[]string
		cmd_line		string
		err 			tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_API_EXEC
	nodes, err.Err = cmd.Flags().GetStringSlice("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	targets, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	target_groups, err.Err = cmd.Flags().GetStringSlice("group")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	target_tags, err.Err = cmd.Flags().GetStringSlice("tag")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	cmd_line, err.Err = cmd.Flags().GetString("cmd")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	if !tree_api.API_INIT(nodes...) {
		fmt.Println("Unable to init api client")
		fmt.Println("Exiting ...")
		return
	}

	var (
		api_cmd		=	tree_api.Command{}
		wait_to_end =	make(chan bool)
	)

	api_cmd.ID = tree_lib.RandomString(20)
	api_cmd.Data = []byte(cmd_line)
	api_cmd.CommandType = tree_api.COMMAND_EXEC

	tree_event.ON(tree_event.ON_CHILD_CONNECTED, func(ev *tree_event.Event){
		path, err := tree_graph.GetPath(string(ev.Data), targets, target_tags, target_groups)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return
		}
		tree_api.SendCommand(&api_cmd, nodes, path, func(e *tree_event.Event, c tree_api.Command)bool{
			fmt.Println(string(c.Data))
			fmt.Println(c.Ended)
			// TODO: End coming faster than other messages FIX !!!!
			if c.Ended {
				return false
			}
			return true
		})
		wait_to_end <- true
	})

	<- wait_to_end
}

func ListInfos(cmd *cobra.Command, args []string){
	var (
		err				tree_lib.TreeError
		nodes			[]string
		node_infos		[]string
	)
	err.From = tree_lib.FROM_LIST_INFOS
	nodes, err.Err = cmd.Flags().GetStringSlice("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	node_infos, err.Err = cmd.Flags().GetStringSlice("node_info")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	if !tree_api.API_INIT(nodes...) {
		fmt.Println("Unable to init api client")
		fmt.Println("Exiting ...")
		return
	}
	var (
		api_cmd =		tree_api.Command{}
	)
	api_cmd.ID = tree_lib.RandomString(20)
	api_cmd.Data = []byte(strings.Join(node_infos,","))
	api_cmd.CommandType = tree_api.COMMAND_LIST

	tree_event.ON(tree_event.ON_CHILD_CONNECTED,func (ev tree_event.Event){
		var path path.Path
		tree_api.SendCommand(&api_cmd, nodes, path, func (e tree_event.Event,c tree_api.Command)bool {
			var (
				err 			tree_lib.TreeError
				info = 			make(map[string]node_info.NodeInfo)
			)
			err.Err = ffjson.Unmarshal(c.Data, &info)
			if !err.IsNull() {
				tree_log.Error(err.From, err.Error())
				return
			}
			for _, a := range info {
				fmt.Println("name: ",a.Name,", Adress: ", a.TreeIp, ":", a.TreePort,", Value: ", a.Value)
			}
			return true
		})
	})
}

func UpdateInfo(cmd *cobra.Command, args []string) {
	var (
		nodes					[]string
		err 					tree_lib.TreeError
		target					string
		ip 						string
		port					int
		add_to_group			string
		delete_from_group		string
		add_to_tag 				string
		delete_from_tag			string
		delete_child			string
		add_child				string
		info					node_info.NodeInfo
	)

	nodes, err.Err = cmd.Flags().GetStringSlice("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetString("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	ip, err.Err = cmd.Flags().GetString("ip")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	port, err.Err = cmd.Flags().GetInt("port")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	add_to_group, err.Err = cmd.Flags().GetString("add_group")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	delete_from_group, err.Err = cmd.Flags().GetString("delete_group")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	add_to_tag, err.Err = cmd.Flags().GetString("add_tag")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	delete_from_tag, err.Err = cmd.Flags().GetString("delete_tag")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	add_child, err.Err = cmd.Flags().GetString("add_child")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	delete_child, err.Err = cmd.Flags().GetString("delete_child")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	if !tree_api.API_INIT(nodes...) {
		fmt.Println("Unable to init api client")
		fmt.Println("Exiting ...")
		return
	}
	info.Name = target
	info.TreeIp = ip
	info.TreePort = port
	info.Childs = append(info.Childs, add_child, delete_child)
	info.Groups = append(info.Groups, add_to_group, delete_from_group)
	info.Tags = append(info.Tags, add_to_tag, delete_from_tag)

	var (
		api_cmd =		tree_api.Command{}
	)

	api_cmd.ID = tree_lib.RandomString(20)
	api_cmd.Data, err.Err = ffjson.Marshal(info)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	api_cmd.CommandType = tree_api.COMMAND_UPDATE

	tree_event.ON(tree_event.ON_CHILD_CONNECTED,func (ev tree_event.Event) {
		var path path.Path
		tree_api.SendCommand(&api_cmd, nodes, path, func(e tree_event.Event, c tree_api.Command) bool {

			return true
		})
	})
}
