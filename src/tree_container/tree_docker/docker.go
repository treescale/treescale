package tree_container

import (
	"github.com/fsouza/go-dockerclient"
	"tree_event"
)

type ContainerInfo struct {
	ID 					string 					`json:"id" toml:"id" yaml:"id"`  // container ID
	Image 				string 					`json:"image" toml:"image" yaml:"image"`   // Container Image Name
	InspectContainer 	*docker.Container 		`json:"inspect" toml:"inspect" yaml:"inspect"`
}

type ImageInfo struct {
	ID 			string              `json:"id" toml:"id" yaml:"id"`
	Name 		string              `json:"name" toml:"name" yaml:"name"`  // Name is a combination of repository:tag from Docker
	Inspect 	*docker.Image       `json:"inspect" toml:"inspect" yaml:"inspect"`
}

var (
	DockerClient 		*docker.Client
	DockerEndpoint = 	"unix:///var/run/docker.sock"
)

func InitDockerClient() (err error) {
	if DockerClient != nil {
		return
	}
	DockerClient, err = docker.NewClient(DockerEndpoint)
	if err != nil {
		return
	}
	return
}

func triggerInitEvent() error {
	dock_containers, err := DockerClient.ListContainers(docker.ListContainersOptions{All: false})
	if err != nil {
		return err
	}

	// Triggering event with currently running Docker containers inside
	tree_event.Trigger(&tree_event.Event{Name:tree_event.ON_DOCKER_INIT, LocalVar: dock_containers})
	return nil
}


func StartEventListener() (err error) {
	err = InitDockerClient()
	if err != nil {
		return
	}

	err = triggerInitEvent()
	if err != nil {
		return
	}
	// When function will be returned local event for ending docker client will be triggered
	defer tree_event.Trigger(&tree_event.Event{Name: tree_event.ON_DOCKER_END, LocalVar: nil})

	ev := make(chan *docker.APIEvents)
	err = DockerClient.AddEventListener(ev)
	if err != nil {
		return
	}

	for  {
		err = callEvent(<- ev)
		if err != nil {
			break
		}
	}

	return
}

func callEvent(event *docker.APIEvents) error {
	switch event.Status {
	case "start", "unpouse":
		{
			dock_inspect, err := DockerClient.InspectContainer(event.ID)
			if err != nil {
				return err
			}
			ci := ContainerInfo{InspectContainer:dock_inspect, ID:event.ID, Image:dock_inspect.Config.Image}
			tree_event.Trigger(&tree_event.Event{Name: tree_event.ON_DOCKER_CONTAINER_START, LocalVar: &ci})
		}
	case "die", "kill", "pause":
		{
			// Sending only Container ID if it stopped
			// Sometimes Docker API not giving all info about container after stopping it
			tree_event.Trigger(&tree_event.Event{Name: tree_event.ON_DOCKER_CONTAINER_STOP, LocalVar: event.ID})
		}
	case "pull", "tag":
		{
			inspect, err := DockerClient.InspectImage(event.ID)
			if err != nil {
				return err
			}
			im := ImageInfo{ID:inspect.ID, Name:event.ID, Inspect:inspect}
			tree_event.Trigger(&tree_event.Event{Name: tree_event.ON_DOCKER_IMAGE_CREATE, LocalVar: &im})
		}
	case "untag", "delete":
		{
			inspect, err := DockerClient.InspectImage(event.ID)
			if err != nil {
				return err
			}
			im := ImageInfo{ID:inspect.ID, Name:event.ID, Inspect:inspect}
			tree_event.Trigger(&tree_event.Event{Name: tree_event.ON_DOCKER_IMAGE_DELETE, LocalVar: &im})
		}
	}
	return nil
}