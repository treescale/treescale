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
	"tree_container/tree_docker"
	"tree_event/custom_event"
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
		groups        	[]string
		tags			[]string
		info 			tree_api.Info
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
	groups, err.Err = cmd.Flags().GetStringSlice("group")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	tags, err.Err = cmd.Flags().GetStringSlice("tag")
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
	info.Group = groups
	info.Target = targets
	info.Tag = tags
	api_cmd.Data, err.Err = ffjson.Marshal(info)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	api_cmd.ID = tree_lib.RandomString(20)
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
				fmt.Println("groups:",a.Groups)
				fmt.Println("tags:",a.Tags)
				fmt.Println("childs",a.Childs)
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
	err.From = tree_lib.FROM_UPDATE_INFO
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

func HandleContStart (cmd *cobra.Command, args []string) {
	var (
		node 		string
		target 		[]string
		image 		string
		command 	string
		ram 		string
		cpu 		string
		container 	string
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_CONT_START
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	image, err.Err = cmd.Flags().GetString("image")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	command, err.Err = cmd.Flags().GetString("command")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	container, err.Err = cmd.Flags().GetString("container")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	ram, err.Err = cmd.Flags().GetString("ram")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	cpu, err.Err = cmd.Flags().GetString("cpu")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	var (
		docker_cmd = 		tree_docker.DockerCmd{}
	)
	docker_cmd.Content = make(map[string]string)
	docker_cmd.Command = tree_docker.COMMAND_DOCKER_CONTAINER_START
	docker_cmd.Content["cmd"] = command
	docker_cmd.Content["ram"] = ram
	docker_cmd.Content["cpu"] = cpu
	docker_cmd.Content["image"] = image
	docker_cmd.Content["container"] = container
	SendDockerCommand(docker_cmd, node, target)
}

func HandleStop (cmd *cobra.Command, args []string) {
	var (
		node 		string
		target 		[]string
		time 		string
		container 	string
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_STOP
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	container, err.Err = cmd.Flags().GetString("container")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	time, err.Err = cmd.Flags().GetString("time")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	var (
		docker_cmd = 		tree_docker.DockerCmd{}
	)
	docker_cmd.Content = make(map[string]string)
	docker_cmd.Command = tree_docker.COMMAND_DOCKER_CONTAINER_STOP
	docker_cmd.Content["timeout"] = time
	docker_cmd.Content["container"] = container
	SendDockerCommand(docker_cmd, node, target)
}

func HandleCreate (cmd *cobra.Command, args []string) {
	var (
		node 		string
		target 		[]string
		image 		string
		command 	string
		ram 		string
		cpu 		string
		container 	string
		count 		string
		start 		bool
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_CREATE
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	image, err.Err = cmd.Flags().GetString("image")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	command, err.Err = cmd.Flags().GetString("command")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	container, err.Err = cmd.Flags().GetString("container")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	ram, err.Err = cmd.Flags().GetString("ram")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	cpu, err.Err = cmd.Flags().GetString("cpu")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	count, err.Err = cmd.Flags().GetString("count")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	start, err.Err = cmd.Flags().GetBool("start")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	var (
		docker_cmd = 		tree_docker.DockerCmd{}
	)
	docker_cmd.Content = make(map[string]string)
	docker_cmd.Command = tree_docker.COMMAND_DOCKER_CONTAINER_CREATE
	docker_cmd.Content["cmd"] = command
	docker_cmd.Content["ram"] = ram
	docker_cmd.Content["cpu"] = cpu
	docker_cmd.Content["image"] = image
	docker_cmd.Content["container"] = container
	docker_cmd.Content["count"] = count
	if start {
		docker_cmd.Content["start"] = "yes"
	} else {docker_cmd.Content["start"] = "no"}

	SendDockerCommand(docker_cmd, node, target)
}

func HandlePause (cmd *cobra.Command, args []string) {
	var (
		node 		string
		target 		[]string
		container 	string
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_PAUSE
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	container, err.Err = cmd.Flags().GetString("container")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	var (
		docker_cmd = 		tree_docker.DockerCmd{}
	)
	docker_cmd.Content = make(map[string]string)
	docker_cmd.Command = tree_docker.COMMAND_DOCKER_CONTAINER_PAUSE
	docker_cmd.Content["container"] = container
	SendDockerCommand(docker_cmd, node, target)
}

func HandleResume (cmd *cobra.Command, args []string) {
	var (
		node 		string
		target 		[]string
		container 	string
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_RESUME
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	container, err.Err = cmd.Flags().GetString("container")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	var (
		docker_cmd = 		tree_docker.DockerCmd{}
	)
	docker_cmd.Content = make(map[string]string)
	docker_cmd.Command = tree_docker.COMMAND_DOCKER_CONTAINER_RESUME
	docker_cmd.Content["container"] = container
	SendDockerCommand(docker_cmd, node, target)
}

func HandleDelete (cmd *cobra.Command, args []string) {
	var (
		node 		string
		target 		[]string
		container 	string
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_DELETE
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	container, err.Err = cmd.Flags().GetString("container")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	var (
		docker_cmd = 		tree_docker.DockerCmd{}
	)
	docker_cmd.Content = make(map[string]string)
	docker_cmd.Command = tree_docker.COMMAND_DOCKER_CONTAINER_DELETE
	docker_cmd.Content["container"] = container
	SendDockerCommand(docker_cmd, node, target)
}

func HandleInspect (cmd *cobra.Command, args []string) {
	var (
		node 		string
		target 		[]string
		container 	string
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_INSPECT
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	container, err.Err = cmd.Flags().GetString("container")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	var (
		docker_cmd = 		tree_docker.DockerCmd{}
	)
	docker_cmd.Content = make(map[string]string)
	docker_cmd.Command = tree_docker.COMMAND_DOCKER_CONTAINER_INSPECT
	docker_cmd.Content["container"] = container
	SendDockerCommand(docker_cmd, node, target)
}

func HandleList (cmd *cobra.Command, args []string) {
	var (
		node 		string
		target 		[]string
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_LIST
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	var (
		docker_cmd = 		tree_docker.DockerCmd{}
	)
	docker_cmd.Content = make(map[string]string)
	if cmd.Parent().Name() == "container" {
		docker_cmd.Command = tree_docker.COMMAND_DOCKER_CONTAINER_LIST
	} else {docker_cmd.Command = tree_docker.COMMAND_DOCKER_IMAGE_LIST}
	docker_cmd.Content["all"] = "yes"
	SendDockerCommand(docker_cmd, node, target)
}

func HandleImageDelete (cmd *cobra.Command, args []string) {
	var (
		node 		string
		target 		[]string
		force 		bool
		image 		string
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_IMAGE_DELETE
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	image, err.Err = cmd.Flags().GetString("image")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	force, err.Err = cmd.Flags().GetBool("force")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	var (
		docker_cmd = 		tree_docker.DockerCmd{}
	)
	docker_cmd.Content = make(map[string]string)
	docker_cmd.Command = tree_docker.COMMAND_DOCKER_IMAGE_DELETE
	docker_cmd.Content["image"] = image
	if force {
		docker_cmd.Content["force"] = "yes"
	} else {docker_cmd.Content["force"] = "no"}
	SendDockerCommand(docker_cmd, node, target)
}

func HandleImagePull (cmd *cobra.Command, args []string) {
	var (
		node 		string
		target 		[]string
		image 		string
		registry 	string
		username 	string
		address		string
		password 	string
		email 		string
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_IMAGE_PULL
	node, err.Err = cmd.Flags().GetString("node")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	target, err.Err = cmd.Flags().GetStringSlice("target")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	registry, err.Err = cmd.Flags().GetString("registry")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	image, err.Err = cmd.Flags().GetString("image")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	address, err.Err = cmd.Flags().GetString("address")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	password, err.Err = cmd.Flags().GetString("password")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	username, err.Err = cmd.Flags().GetString("username")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	email, err.Err = cmd.Flags().GetString("email")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	var (
		docker_cmd = 		tree_docker.DockerCmd{}
	)
	docker_cmd.Content = make(map[string]string)
	docker_cmd.Command = tree_docker.COMMAND_DOCKER_IMAGE_PULL
	docker_cmd.Content["image"] = image
	docker_cmd.Content["registry"] = registry
	docker_cmd.Content["registry_username"] = username
	docker_cmd.Content["registry_password"] = password
	docker_cmd.Content["registry_address"] = address
	docker_cmd.Content["registry_email"] = email
	SendDockerCommand(docker_cmd, node, target)
}

func SendDockerCommand (cmd tree_docker.DockerCmd, node string, target []string){
	var err tree_lib.TreeError
	if !tree_api.API_INIT(node) {
		fmt.Println("Unable to init api client")
		fmt.Println("Exiting ...")
		return
	}
	err.From = tree_lib.FROM_SEND_DOCKER_COMMAND
	var (
		api_cmd =		tree_api.Command{}
		wait = 			make(chan bool)
	)

	api_cmd.ID = tree_lib.RandomString(20)
	api_cmd.Data, err.Err = ffjson.Marshal(cmd)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	api_cmd.CommandType = tree_api.COMMAND_CONTAINER

	tree_event.ON(tree_event.ON_CHILD_CONNECTED,func (ev *tree_event.Event) {
		path := &tree_graph.Path{From: node, Nodes: target }
		tree_api.SendCommand(&api_cmd, path, func(e *tree_event.Event, c tree_api.Command) bool {
			fmt.Println(string(c.Data))
			fmt.Println(c.Ended)
			if c.Ended {
				return false
			}
			return true
		})
		wait <- true
	})
	<- wait
}

func SendAddEventHandlerCommand(cmd *cobra.Command, args []string) {
	var (
		event_name		string
		err 			tree_lib.TreeError
		handler			custom_event.Handler
		node 			string
		targets 		[]string
		target_groups 	[]string
		target_tags 	[]string
	)

	err.From = tree_lib.FROM_SEND_COMMAND

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



	event_name, err.Err = cmd.Flags().GetString("event")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	handler.Handler, err.Err = cmd.Flags().GetString("handler")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	handler.IsFile, err.Err = cmd.Flags().GetBool("file")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	handler.ExecUser, err.Err = cmd.Flags().GetString("user")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	handler.Password, err.Err = cmd.Flags().GetString("pass")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	var (
		api_cmd	= tree_api.Command{}
		wait_to_end =	make(chan bool)
	)
	api_cmd.Data, err.Err = ffjson.Marshal(map[string]interface{}{
		"name": event_name,
		"handlers": []custom_event.Handler{handler},
	})
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	if !tree_api.API_INIT(node) {
		fmt.Println("Unable to init api client")
		fmt.Println("Exiting ...")
		return
	}

	api_cmd.ID = tree_lib.RandomString(20)
	api_cmd.CommandType = tree_api.COMMAND_ADD_CUSTOM_EVENT

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


func SendEventTriggerCommand(cmd *cobra.Command, args []string) {
	var (
		event_name		string
		err 			tree_lib.TreeError
		node 			string
		targets 		[]string
		target_groups 	[]string
		target_tags 	[]string
	)

	err.From = tree_lib.FROM_SEND_COMMAND

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



	event_name, err.Err = cmd.Flags().GetString("event")
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
		api_cmd	= tree_api.Command{}
		wait_to_end =	make(chan bool)
	)
	api_cmd.Data = []byte(event_name)
	api_cmd.ID = tree_lib.RandomString(20)
	api_cmd.CommandType = tree_api.COMMAND_TRIGGER_EVENT

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