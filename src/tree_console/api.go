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
	"strings"
)


func HandleApiExec(cmd *cobra.Command, args []string) {
	var (
		node 			string
		targets 		[]string
		target_groups 	[]string
		target_tags 	[]string
		cmd_line		string
		err 			tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_API_EXEC
	node, err.Err = cmd.Flags().GetString("node")
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

	if !tree_api.API_INIT(node) {
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
		path := &tree_graph.Path{From: node, Nodes: targets, Tags: target_tags, Groups: target_groups }

		tree_api.SendCommand(&api_cmd, path, func(e *tree_event.Event, c tree_api.Command)bool{
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
		node			string
		targets			[]string
	)
	err.From = tree_lib.FROM_LIST_INFOS
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	targets, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	if !tree_api.API_INIT(node) {
		fmt.Println("Unable to init api client")
		fmt.Println("Exiting ...")
		return
	}
	var (
		api_cmd =		tree_api.Command{}
		wait = 			make(chan bool)
	)
	api_cmd.ID = tree_lib.RandomString(20)
	api_cmd.Data = []byte(strings.Join(targets,","))
	api_cmd.CommandType = tree_api.COMMAND_LIST

	tree_event.ON(tree_event.ON_CHILD_CONNECTED,func (ev *tree_event.Event){
		path := &tree_graph.Path{From: node, Nodes: []string{node} }

		tree_api.SendCommand(&api_cmd, path, func (e *tree_event.Event,c tree_api.Command)bool {
			var (
				err 			tree_lib.TreeError
				info = 			make(map[string]node_info.NodeInfo)
			)
			err.Err = ffjson.Unmarshal(c.Data, &info)
			if !err.IsNull() {
				tree_log.Error(err.From, err.Error())
				return false
			}
			for _, a := range info {
				fmt.Println("name: ",a.Name,", Adress: ", a.TreeIp, ":", a.TreePort,", Value: ", a.Value)
			}
			return false
		})
		wait <- true
	})
	<- wait
}

func UpdateInfo(cmd *cobra.Command, args []string) {
	var (
		node					string
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

	node, err.Err = cmd.Flags().GetString("node")
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
	if !tree_api.API_INIT(node) {
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
		wait = 			make(chan bool)
	)

	api_cmd.ID = tree_lib.RandomString(20)
	api_cmd.Data, err.Err = ffjson.Marshal(info)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	api_cmd.CommandType = tree_api.COMMAND_UPDATE

	tree_event.ON(tree_event.ON_CHILD_CONNECTED,func (ev *tree_event.Event) {
		path := &tree_graph.Path{From: node, Nodes: []string{node} }
		tree_api.SendCommand(&api_cmd, path, func(e *tree_event.Event, c tree_api.Command) bool {

			return false
		})
		wait <- true
	})
	<- wait
}
