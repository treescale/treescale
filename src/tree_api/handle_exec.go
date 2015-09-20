package tree_api
import (
	"tree_event"
	"strings"
	"os/exec"
	"tree_log"
)


const (
	log_from_handle_exec	=	"Handle Exec command"
)

// Executing some commands using exec.Command functionality from Go in OS
func HandleExecCommand(e *tree_event.Event, cmd Command) {
	var (
		out			=	&WriterCallback{BufferMaxSize: 1024}
		cmd_str		=	string(cmd.Data)
		cmd_options	= 	strings.Split(cmd_str, " ")
		cmd			=	exec.Command(cmd_options[0], cmd_options[1:]...)
		err 			error
	)

	defer out.End()

	cmd.Stdout = out
	cmd.Stdin = out
	err = cmd.Run()
	if err != nil {
		tree_log.Error(log_from_handle_exec, err.Error())
	}
}