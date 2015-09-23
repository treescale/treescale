package tree_docker

import (
	"github.com/fsouza/go-dockerclient"
	"io"
	"fmt"
	"strconv"
	"github.com/pquerna/ffjson/ffjson"
	"strings"
	"tree_lib"
)


const (
	COMMAND_DOCKER_CONTAINER_CREATE = 1
	COMMAND_DOCKER_CONTAINER_START = 2
	COMMAND_DOCKER_CONTAINER_STOP = 3
	COMMAND_DOCKER_CONTAINER_PAUSE = 4
	COMMAND_DOCKER_CONTAINER_RESUME = 5
	COMMAND_DOCKER_CONTAINER_DELETE = 6
	COMMAND_DOCKER_CONTAINER_INSPECT = 7

	COMMAND_DOCKER_IMAGE_PULL = 8
	COMMAND_DOCKER_IMAGE_DELETE = 9

	COMMAND_DOCKER_CONTAINER_LIST = 10
	COMMAND_DOCKER_IMAGE_LIST = 11
)

type DockerCmd struct {
	Command		int					`json:"command" toml:"command" yaml:"command"`
	Content		map[string]string	`json:"content" toml:"content" yaml:"content"`  // string -> string map for getting command data
}

type DockerCmdOutput struct {
	Error		bool
	Message		string
	Data		map[string]string
	DataList	map[string]interface{}
}

func writeOutput(out io.Writer, is_error bool, message string, data map[string]string) {
	dc_out := &DockerCmdOutput{Error:is_error, Message:message, Data:data}
	send_byte, _ := ffjson.Marshal(dc_out)
	out.Write(send_byte)
}

func writeDcOutput(out io.Writer, dc_out DockerCmdOutput) {
	send_byte, _ := ffjson.Marshal(dc_out)
	out.Write(send_byte)
}

func HandleApiCommand(cmd_data []byte, api_out io.Writer) {
	var (
		err tree_lib.TreeError
		cmd DockerCmd
	)
	err.From = tree_lib.FROM_HANDLE_API_COMMAND
	err.Err = ffjson.Unmarshal(cmd_data, &cmd)
	if !err.IsNull() {
		writeOutput(api_out, true, fmt.Sprintf("--- Unable to parse Container command data: %s", string(cmd_data)), map[string]string{})
		return
	}

	ContainerCommands(&cmd, api_out)
	fmt.Println(string(cmd_data))
}

func ContainerCommands(cmd *DockerCmd, out io.Writer) {
	var (
		err tree_lib.TreeError
	)
	err.From = tree_lib.FROM_CONTAINER_COMMANDS
	switch cmd.Command {
	case COMMAND_DOCKER_CONTAINER_CREATE, COMMAND_DOCKER_CONTAINER_START:
		{
			var (
				cont 		*docker.Container
				conf 	=	&docker.Config{}
				host_conf =	&docker.HostConfig{}
				cont_count 	int
				start_cont = false
			)
			if cc, ok := cmd.Content["count"]; ok {
				cont_count, err.Err = strconv.Atoi(cc)
				if !err.IsNull() {
					writeOutput(out, true, fmt.Sprintf("--- Invalid number given for Containers count: %s", cc), cmd.Content)
					return
				}
			} else {
				cont_count = 1
			}

			if run_cmd, ok := cmd.Content["cmd"]; ok {
				conf.Cmd = []string{run_cmd}
			}

			if img, ok := cmd.Content["image"]; ok {
				conf.Image = img
			}

			// By default we just don't need any output
			conf.AttachStderr = false
			conf.AttachStdin = false
			conf.AttachStdout = false

			if cs, ok := cmd.Content["cpu"]; ok {
				var cs_int int
				cs_int, err.Err = strconv.Atoi(cs)
				if !err.IsNull() {
					writeOutput(out, true, fmt.Sprintf("--- Invalid number given for CPU Shares: %s", cs), cmd.Content)
					return
				}
				host_conf.CPUShares = int64(cs_int)
			}

			if ram, ok := cmd.Content["ram"]; ok {
				var ram_int int
				ram_int, err.Err = strconv.Atoi(ram)
				if !err.IsNull() {
					writeOutput(out, true, fmt.Sprintf("--- Invalid number given for RAM: %s", ram), cmd.Content)
					return
				}
				host_conf.Memory = int64(ram_int)
			}

			if st, ok := cmd.Content["start"]; ok {
				switch st {
				case "yes", "y", "true", "t":
					start_cont = true
				case "no", "n", "false", "f":
					start_cont = false
				}
			}

			// If we just want to start container by ID just running it and returning
			if cmd.Command == COMMAND_DOCKER_CONTAINER_START {
				if cid, ok := cmd.Content["container"]; ok {
					err.Err = DockerClient.StartContainer(cid, host_conf)
					if !err.IsNull() {
						writeOutput(out, true, fmt.Sprintf("--- Unable to start container: %s", err.Error()), cmd.Content)
						return
					}
					writeOutput(out, false, fmt.Sprintf("Container Started \n  Container -> %s\n", cid), cmd.Content)
				}
				return
			}

			for i := 0; i < cont_count; i++ {
				cont, err.Err = DockerClient.CreateContainer(docker.CreateContainerOptions{
					Config: conf,
					HostConfig: host_conf,
				})

				if !err.IsNull() {
					writeOutput(out, true, fmt.Sprintf("--- Container creation error: %s", err.Error()), cmd.Content)
					return
				}
				writeOutput(out, false, fmt.Sprintf("Container Created \n  ID -> %s\n  Name -> %s\n", cont.ID, cont.Name), cmd.Content)
				if start_cont {
					err.Err = DockerClient.StartContainer(cont.ID, host_conf)
					if !err.IsNull() {
						writeOutput(out, true, fmt.Sprintf("--- Unable to start container: %s", err.Error()), cmd.Content)
						return
					}
					writeOutput(out, false, fmt.Sprintf("Container Started \n  Name -> %s\n", cont.Name), cmd.Content)
				}
			}
		}
	case COMMAND_DOCKER_CONTAINER_PAUSE:
		{
			if cid, ok := cmd.Content["container"]; ok {
				err.Err = DockerClient.PauseContainer(cid)
				if !err.IsNull() {
					writeOutput(out, true, fmt.Sprintf("--- Unable to pause container: %s", err.Error()), cmd.Content)
					return
				}
				writeOutput(out, false, fmt.Sprintf("Container Paused \n  Container -> %s\n", cid), cmd.Content)
			}
		}
	case COMMAND_DOCKER_CONTAINER_RESUME:
		{
			if cid, ok := cmd.Content["container"]; ok {
				err.Err = DockerClient.UnpauseContainer(cid)
				if !err.IsNull() {
					writeOutput(out, true, fmt.Sprintf("--- Unable to Resume container: %s", err.Error()), cmd.Content)
					return
				}
				writeOutput(out, false, fmt.Sprintf("Container Resumed \n  Container -> %s\n", cid), cmd.Content)
			}
		}
	case COMMAND_DOCKER_CONTAINER_DELETE:
		{
			if cid, ok := cmd.Content["container"]; ok {
				DockerClient.StopContainer(cid, 0)  // Stopping container if it exists
				err.Err = DockerClient.RemoveContainer(docker.RemoveContainerOptions{ID: cid, Force: true})
				if !err.IsNull() {
					writeOutput(out, true, fmt.Sprintf("--- Unable to Resume container: %s", err.Error()), cmd.Content)
					return
				}
				writeOutput(out, false, fmt.Sprintf("Container Resumed \n  Container -> %s\n", cid), cmd.Content)
			}
		}
	case COMMAND_DOCKER_CONTAINER_STOP:
		{
			var (
				stop_timeout  = uint(0)
				st_int			int
			)

			if tm, ok := cmd.Content["timeout"]; ok {
				st_int, err.Err = strconv.Atoi(tm)
				if !err.IsNull() {
					writeOutput(out, true, fmt.Sprintf("--- Invalid number given for Timeout: %s", tm), cmd.Content)
					return
				}
				stop_timeout = uint(st_int)
			}

			if cid, ok := cmd.Content["container"]; ok {
				err.Err = DockerClient.StopContainer(cid, stop_timeout)
				if !err.IsNull() {
					writeOutput(out, true, fmt.Sprintf("--- Unable to Stop container: %s", err.Error()), cmd.Content)
					return
				}
				writeOutput(out, false, fmt.Sprintf("Container Stopped \n  Container -> %s\n", cid), cmd.Content)
			}
		}

	case COMMAND_DOCKER_IMAGE_PULL:
		{
			var (
				registry 				string
				repository 				string
				tag						string
				image					string
				img_repo				string
				registery_username		string
				registery_email			string
				registery_passord		string
				registery_address		string
			)

			if r, ok := cmd.Content["registry"]; ok {
				registry = r
			} else {
				registry = ""
			}

			if im, ok := cmd.Content["image"]; ok {
				image = im
			} else {
				writeOutput(out, true, "--- Image Name is required during image pull process", cmd.Content)
				return
			}

			str_split := strings.Split(image, ":")
			if len(str_split) != 2 {
				writeOutput(out, true, "--- Please Specify image tag name with this format <repository>:<tag>", cmd.Content)
				return
			}

			repository = str_split[0]
			tag = str_split[1]
			img_repo = fmt.Sprintf("%s/%s", registry, repository)

			// Getting Registery authentication data
			if ru, ok := cmd.Content["registry_username"]; ok {
				registery_username = ru
			}
			if rp, ok := cmd.Content["registry_password"]; ok {
				registery_passord = rp
			}
			if rem, ok := cmd.Content["registry_email"]; ok {
				registery_email = rem
			}
			if raddr, ok := cmd.Content["registry_address"]; ok {
				registery_address = raddr
			}

			err.Err = DockerClient.PullImage(docker.PullImageOptions{
				Registry: registry,
				Repository: img_repo,
				Tag: tag,
				OutputStream: nil,
			}, docker.AuthConfiguration{
				Username: registery_username,
				Email: registery_email,
				Password: registery_passord,
				ServerAddress: registery_address,
			})

			if !err.IsNull() {
				writeOutput(out, true, fmt.Sprintf("--- Pulling image error: %s", err.Error()), cmd.Content)
				return
			}

			err.Err = DockerClient.TagImage(fmt.Sprintf("%s:%s", img_repo, tag), docker.TagImageOptions{
				Repo: repository,
				Tag: tag,
				Force: true,
			})

			if !err.IsNull() {
				writeOutput(out, true, fmt.Sprintf("--- Error Renaming pulled image %s : %s", fmt.Sprintf("%s:%s", img_repo, tag), err.Error()), cmd.Content)
				DockerClient.RemoveImageExtended(fmt.Sprintf("%s:%s", img_repo, tag), docker.RemoveImageOptions{Force:true})
				return
			}

			DockerClient.RemoveImage(fmt.Sprintf("%s:%s", img_repo, tag))
			writeOutput(out, false, fmt.Sprintf("Image Created %s", image), cmd.Content)
		}
	case COMMAND_DOCKER_IMAGE_DELETE:
		{
			var (
				image 		string
				force	=	false
			)

			if im, ok := cmd.Content["image"]; ok {
				image = im
			} else {
				writeOutput(out, true, "--- For Deleting image you need to specifi image name", cmd.Content)
				return
			}

			if f, ok := cmd.Content["force"]; ok {
				switch f {
				case "yes", "y", "true", "t":
					force = true
				case "no", "n", "false", "f":
					force = false
				}
			}

			err.Err = DockerClient.RemoveImageExtended(image, docker.RemoveImageOptions{Force:force})

			if !err.IsNull() {
				writeOutput(out, true, fmt.Sprintf("--- Error Deleting image %s : %s", image, err.Error()), cmd.Content)
				return
			}

			writeOutput(out, true, fmt.Sprintf("Image deleted %s", image), cmd.Content)
		}


	case COMMAND_DOCKER_CONTAINER_INSPECT:
		{
			var (
				cont_id			string
				container	=	make(map[string]interface{})
			)

			if cid, ok := cmd.Content["container"]; ok {
				cont_id = cid
			} else {
				writeOutput(out, true, "--- Container Name or ID is required during inspecting", cmd.Content)
				return
			}

			container[cont_id], err.Err = DockerClient.InspectContainer(cont_id)
			writeDcOutput(out, DockerCmdOutput{
				Error: false,
				Message: "Container Inspected Successfully",
				Data: cmd.Content,
				DataList: container,
			})
		}


	case COMMAND_DOCKER_CONTAINER_LIST:
		{
			var (
				all			=	false
				containers		[]docker.APIContainers
				w_list		=	make(map[string]interface{})
			)

			if a, ok := cmd.Content["all"]; ok {
				switch a {
				case "yes", "y", "true", "t":
					all = true
				case "no", "n", "false", "f":
					all = false
				}
			}

			containers, err.Err = DockerClient.ListContainers(docker.ListContainersOptions{All:all})
			if !err.IsNull() {
				writeOutput(out, true, "--- Error Getting Container list", cmd.Content)
				return
			}

			for _, c :=range containers {
				w_list[c.ID], err.Err = DockerClient.InspectContainer(c.ID)
				if !err.IsNull() {
					writeDcOutput(out, DockerCmdOutput{
						Error: true,
						Message: fmt.Sprintf("--- Error Inspecting container %s", c.ID),
						Data: cmd.Content,
						DataList: w_list,
					})
				}
			}

			writeDcOutput(out, DockerCmdOutput{
				Error: false,
				Message: "Containers list fetched successfully",
				Data: cmd.Content,
				DataList: w_list,
			})
		}
	case COMMAND_DOCKER_IMAGE_LIST:
		{
			var (
				all			=	false
				images			[]docker.APIImages
				w_list		=	make(map[string]interface{})
			)

			if a, ok := cmd.Content["all"]; ok {
				switch a {
				case "yes", "y", "true", "t":
					all = true
				case "no", "n", "false", "f":
					all = false
				}
			}

			images, err.Err = DockerClient.ListImages(docker.ListImagesOptions{All:all})
			if !err.IsNull() {
				writeOutput(out, true, "--- Error Getting Image list", cmd.Content)
				return
			}

			for _, im :=range images {
				w_list[im.ID], err.Err = DockerClient.InspectImage(im.ID)
				if !err.IsNull() {
					writeDcOutput(out, DockerCmdOutput{
						Error: true,
						Message: fmt.Sprintf("--- Error Inspecting image %s", im.ID),
						Data: cmd.Content,
						DataList: w_list,
					})
				}
			}

			writeDcOutput(out, DockerCmdOutput{
				Error: false,
				Message: "Images list fetched successfully",
				Data: cmd.Content,
				DataList: w_list,
			})
		}
	}
}