package tree_console

import (
	tree_path "tree_graph/path"
	"github.com/spf13/cobra"
	"tree_log"
	"fmt"
	"tree_api"
	"tree_event"
	"tree_lib"
)


func HandleApiExec(cmd *cobra.Command, args []string) {
	var (
		nodes 			[]string
		cmd_line		string
		err 			tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_API_EXEC
	nodes, err.Err = cmd.Flags().GetStringSlice("nodes")
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
		path		=	tree_path.Path{}
		wait_to_end =	make(chan bool)
	)

	api_cmd.ID = tree_lib.RandomString(20)
	api_cmd.Data = []byte(cmd_line)
	api_cmd.CommandType = tree_api.COMMAND_EXEC

	tree_event.ON(tree_event.ON_CHILD_CONNECTED, func(ev *tree_event.Event){
		fmt.Println(string(ev.Data))
		tree_api.SendCommand(&api_cmd, nodes, &path, func(e *tree_event.Event, c tree_api.Command)bool{
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