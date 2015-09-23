package tree_balancer

import (
	"tree_container/tree_docker"
	"github.com/fsouza/go-dockerclient"
	"tree_event"
	"tree_log"
	"tree_lib"
)

const (
	log_from_balancer = "Balancer"
)

var (
	AvailableServices = make(map[string]BalancingService)   // IP:Port -> BalancingService map
	containerAddressMap = make(map[string]Address)		// Keeping container and Address for deleting
)

func AddService(addr string, alg string) (err tree_lib.TreeError) {
	err.From = tree_lib.FROM_ADD_SERVICE
	if _, ok := AvailableServices[addr]; ok {
		return
	}
	var a Address
	a, err.Err = AddressFromString(addr)
	if !err.IsNull() {
		return
	}
	AvailableServices[addr], err.Err = NewBalancer(a, alg)
	return
}

func DropService(addr string) (err tree_lib.TreeError) {
	err.From = tree_lib.FROM_DROP_SERVICE
	if s, ok := AvailableServices[addr]; ok {
		err.Err = s.DropService()
		return
	}
	return
}

func (bs *BalancingService) SubscribeEvents() {
	tree_event.ON(tree_event.ON_DOCKER_INIT, func(e *tree_event.Event){
		var err tree_lib.TreeError
		err.From = tree_lib.FROM_SUBSCRIBE_EVENTS
		if e.LocalVar == nil {
			tree_log.Info(err.From, "Containers list is nil during INIT event")
			return
		}

		for _, c := range e.LocalVar.([]docker.APIContainers) {
			if port, ok := bs.DockerImages[c.Image]; ok {
				var ci *docker.Container
				ci, err.Err = tree_docker.DockerClient.InspectContainer(c.ID)
				if !err.IsNull() {
					continue
				}
				cont_addr := Address{IP:ci.NetworkSettings.IPAddress, Port:port}
				err.Err = bs.AddDestination(cont_addr)
				if !err.IsNull() {
					return
				}
				containerAddressMap[c.ID] = cont_addr
			}
		}
	})
	tree_event.ON(tree_event.ON_DOCKER_CONTAINER_START, func(e *tree_event.Event){
		var err tree_lib.TreeError
		if e.LocalVar == nil {
			tree_log.Info(log_from_balancer, "Container Info is nil during container Start event")
			return
		}

		ci := e.LocalVar.(*tree_docker.ContainerInfo)
		if port, ok := bs.DockerImages[ci.Image]; ok {
			cont_addr := Address{IP:ci.InspectContainer.NetworkSettings.IPAddress, Port:port}
			err.Err = bs.AddDestination(cont_addr)
			if !err.IsNull() {
				return
			}
			containerAddressMap[ci.ID] = cont_addr
		}
	})

	tree_event.ON(tree_event.ON_DOCKER_CONTAINER_STOP, func(e *tree_event.Event){
		if e.LocalVar == nil {
			tree_log.Info(log_from_balancer, "Container ID is nil during container Stop event")
			return
		}
		cont_id := e.LocalVar.(string)
		if cont_addr, ok := containerAddressMap[cont_id]; ok {
			bs.DeleteDestination(cont_addr)
			delete(containerAddressMap, cont_id)
			bs.CheckForStopEvent()
		}
	})
}

func (bs *BalancingService) CheckForStopEvent() (err tree_lib.TreeError) {
	// If our balancer don't have any destination we need to stop it and call global callback about it
	if len(bs.destinations) > 0 {
		return
	}

	tree_event.Trigger(&tree_event.Event{Name: tree_event.ON_BALANCER_SERVICE_STOP, LocalVar: &bs.BalancerConfig})

	// Deleting service from LVS
//	err = bs.DropService()  // We don't need to delete service from here
	return
}