package tree_api
import (
	"tree_event"
	"tree_container/tree_docker"
	"tree_lib"
	"github.com/pquerna/ffjson/ffjson"
	"tree_log"
)

func HandleContainerCommand (ev *tree_event.Event, cmd Command){
	var (
		out					=			&WriterCallback{BufferMaxSize: 1024}
		docker_cmd 			= 			tree_docker.DockerCmd{}
		err 							tree_lib.TreeError
		ev_data							[]byte
	)
	err.From = tree_lib.FROM_HANDLE_CONTAINER_COMMAND
	err.Err = ffjson.Unmarshal(cmd.Data, &docker_cmd)
	if !err.IsNull() {
		tree_log.Error(err.From, "unable to unmarshal command data as a docker command -> ", err.Error())
		return
	}
	out.OutCallback = func(data []byte, ended bool) {
		cb_cmd := cmd
		cb_cmd.Ended = ended
		cb_cmd.Data = data
		ev_data, err.Err = ffjson.Marshal(cb_cmd)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return
		}
		SendCommandCallback(ev, ev_data)
	}


	defer out.End()


	tree_docker.ContainerCommands(&docker_cmd, out)
}