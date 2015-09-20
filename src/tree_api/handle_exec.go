package tree_api
import (
	"tree_event"
	"strings"
	"os/exec"
	"tree_log"
	"github.com/pquerna/ffjson/ffjson"
)


const (
	log_from_handle_exec	=	"Handle Exec command"
)

// Executing some commands using exec.Command functionality from Go in OS
func HandleExecCommand(e *tree_event.Event, api_cmd Command) {
	var (
		out			=	&WriterCallback{BufferMaxSize: 1024}
		cmd_str		=	string(api_cmd.Data)
		cmd_options	= 	strings.Split(cmd_str, " ")
		cmd			=	exec.Command(cmd_options[0], cmd_options[1:]...)
		err 			error
	)

	out.OutCallback = func(data []byte, ended bool) {
		cb_cmd := api_cmd
		cb_cmd.Ended = ended
		cb_cmd.Data = data
		ev_data, err := ffjson.Marshal(cb_cmd)
		if err != nil {
			tree_log.Error(log_from_node_api, err.Error())
			return
		}
		cb_ev := &tree_event.Event{}
		cb_ev.Name = tree_event.ON_API_COMMAND_CALLBACK
		cb_ev.Data = ev_data
		EmitToApi(cb_ev, e.From)
	}

	defer out.End()

	cmd.Stdout = out
	cmd.Stderr = out
	err = cmd.Run()
	if err != nil {
		tree_log.Error(log_from_handle_exec, err.Error())
	}
}