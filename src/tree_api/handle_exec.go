package tree_api
import (
	"tree_event"
	"strings"
	"os/exec"
	"tree_log"
	"github.com/pquerna/ffjson/ffjson"
	"tree_lib"
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
		err 			tree_lib.TreeError
		ev_data			[]byte
	)
	err.From = tree_lib.FROM_HANDLE_EXEC_COMMAND
	out.OutCallback = func(data []byte, ended bool) {
		cb_cmd := api_cmd
		cb_cmd.Ended = ended
		cb_cmd.Data = data
		ev_data, err.Err = ffjson.Marshal(cb_cmd)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return
		}
		cb_ev := &tree_event.EventEmitter{}
		cb_ev.Name = tree_event.ON_API_COMMAND_CALLBACK
		cb_ev.Data = ev_data
		cb_ev.ToNodes = []string{e.From}
		if len(e.FromApi) > 0 {
			cb_ev.ToApi = []string{e.FromApi}
		}
		tree_event.Emit(cb_ev)
	}

	defer out.End()

	cmd.Stdout = out
	cmd.Stderr = out
	err.Err = cmd.Run()
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
	}
}