package tree_balancer

// #include "treelvs.h"
import "C"

import (
	"unsafe"
	"errors"
	"strings"
	"fmt"
	"strconv"
	"tree_event"
)

type Address struct {
	IP 		string		`toml:"ip" json:"ip"`
	Port	int			`toml:"port" json:"port"`
}

func (addr *Address) IsEqual(addr2 Address) bool {
	return (addr.IP == addr2.IP && addr.Port == addr2.Port)
}

func (addr *Address) ToString() string {
	return fmt.Sprintf("%s:%d", addr.IP, addr.Port)
}

func AddressFromString(addr string) (a Address, err error) {
	sp := strings.Split(addr, ":")
	if len(sp) != 2 {
		err = errors.New("Address Format should be IP:PORT !")
		return
	}
	a.IP = sp[0]
	a.Port, err = strconv.Atoi(sp[1])
	return
}


const (
	AlgRoundRubin = "rr"
	AlgWeightedRoundRobin = "wrr"
	AlgLeastConnection = "lc"
	AlgWeightedLeastConnection = "wlc"
	AlgLocalityBasedLeastConnection = "lblc"
	AlgLocalityBasedLeastConnectionWithReplication = "lblcr"
	AlgDestinationHashing = "dh"
	AlgSourceHashing = "sh"
	AlgShortestExpectedDelay = "sed"
	AlgNeverQueue = "nq"
)

type BalancingService struct {
	Address			Address					`toml:"address" json:"address" yaml:"address"`
	LvsService		unsafe.Pointer        	`toml:"-" json:"-" yaml:"-"`
	destinations	[]Address				`toml:"-" json:"-" yaml:"-"`

	// Algorithm for load balancing (rr, wrr, lc, wlc, lblc, lblcr, dh, sh, sed, nq)
	/*
		Round-robin -> rr
		Weighted round-robin -> wrr
		Least-connection -> lc
		Weighted least-connection -> wlc
		Locality-based least-connection -> lblc
		Locality-based least-connection with replication -> lblcr
		Destination hashing -> dh
		Source hashing -> sh
		Shortest expected delay -> sed
		Never queue -> nq
	*/
	Algorithm		string						`toml:"algorithm" json:"algorithm" yaml:"algorithm"`

	// Load Balancing Staff
	DockerImages	map[string]int				`toml:"docker_images" json:"docker_images" yaml:"docker_images"`  // Image Name -> Balancing Port
	ChildServers	map[string]string			`toml:"child_servers" json:"child_servers" yaml:"child_servers"`	// IP -> Balancing Port
	BalancerConfig	BalancerConfig				`toml:"-" json:"-" yaml:"-"`
}


type BalancerConfig struct {
	Name		string				`toml:"name" json:"name" yaml:"name"`
	Server		string				`toml:"server" json:"server" yaml:"server"`				// name of the server for adding balancer for this IP
	IP			string				`toml:"ip" json:"ip" yaml:"ip"`
	Port		int					`toml:"port" json:"port" yaml:"port"`
	Algorithm	string				`toml:"alg" json:"alg" yaml:"alg"`						// Algorithm for load balancing, default rr (Round Rubin)
	// Image List with following format
	// []string{"img_name:tag|port"}
	Images		[]string			`toml:"images" json:"images" yaml:"images"`
}

func NewBalancerFromConfig(bc BalancerConfig) (bs BalancingService, err error) {
	var bs_addr Address
	bs_addr = Address{IP:bc.IP, Port:bc.Port}
	bs, err = NewBalancer(bs_addr, bc.Algorithm)
	if err != nil {
		return
	}

	for _, im :=range bc.Images {
		im = strings.Replace(im, " ", "", -1)  // Deleting spaces
		im_split := strings.Split(im, "|") // splitting image name and port
		bs.DockerImages[im_split[0]], err = strconv.Atoi(im_split[1])
		if err != nil {
			return
		}
	}
	bs.BalancerConfig = bc
	bs.SubscribeEvents()
	tree_event.Trigger(&tree_event.Event{Name: tree_event.ON_BALANCER_SERVICE_START, LocalVar: &bs.BalancerConfig})
	return
}

func NewBalancer(addr Address, algorithm string) (bs BalancingService, err error) {
	bs.DockerImages = make(map[string]int)
	bs.ChildServers = make(map[string]string)
	bs.Algorithm = algorithm
	bs.Address = addr
	err = bs.initLVS()
	if err != nil {
		return
	}
	// Deleting service before we will add it
	// In case if it is registered already
	DropService(addr.ToString())
	err = bs.CreateService()
	if err != nil {
		return
	}

	// If Process is exiting then we need to delete this service before exit
	tree_event.ON(tree_event.ON_PROGRAM_EXIT, func(e *tree_event.Event){
		bs.DropService()
	})
	return
}

func (bs *BalancingService) initLVS() error {
	res := C.init_ipvs()
	if res != 0 {
		error_text := C.GoString(C.ipvs_error())
		return errors.New(error_text)
	}
	res = C.ipvs_flush();
	if res != 0 {
		error_text := C.GoString(C.ipvs_error())
		return errors.New(error_text)
	}
	return nil
}

func (bs *BalancingService) CreateService() error {
	bs.LvsService = C.create_service(C.CString(bs.Address.IP), C.int(bs.Address.Port), C.CString(bs.Algorithm))
	res := C.add_service(bs.LvsService)
	if res != 0 {
		error_text := C.GoString(C.ipvs_error())
		return errors.New(error_text)
	}
	return nil
}

func (bs *BalancingService) DropService() error {
	res := C.remove_service(bs.LvsService)
	if res != 0 {
		error_text := C.GoString(C.ipvs_error())
		return errors.New(error_text)
	}
	return nil
}

func (bs *BalancingService) AddDestination(addrs...Address) error {
	var dst unsafe.Pointer
	for _, d :=range addrs {
		// Checking we have this address or not
		for _, d2 :=range bs.destinations {
			if d2.IsEqual(d) {
				// If there is Address that we already added just exiting from this function without error
				return nil
			}
		}
		dst = C.create_dest(C.CString(d.IP), C.int(d.Port))
		res := C.add_dest(bs.LvsService, dst)
		if res != 0 {
			error_text := C.GoString(C.ipvs_error())
			return errors.New(error_text)
		}
		bs.destinations = append(bs.destinations, d)
	}
	return nil
}

func (bs *BalancingService) DeleteDestination(addrs...Address) error {
	var dst unsafe.Pointer
	for _, d :=range addrs {
		// Checking we have this address or not
		for i, d2 :=range bs.destinations {
			if d2.IsEqual(d) {
				dst = C.create_dest(C.CString(d.IP), C.int(d.Port))
				res := C.remove_dest(bs.LvsService, dst)
				if res != 0 {
					error_text := C.GoString(C.ipvs_error())
					return errors.New(error_text)
				}
				bs.destinations = append(bs.destinations[:i], bs.destinations[i+1:]...)
			}
		}
	}
	return nil
}