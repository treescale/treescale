package tree_node
import (
	"tree_db"
	"tree_node/node_info"
	"tree_log"
	"fmt"
	"tree_net"
	"tree_balancer"
	"github.com/pquerna/ffjson/ffjson"
	"tree_container/tree_docker"
	"time"
	"tree_lib"
	"tree_event"
	"cmd/compile/internal/big"
)


var (
	log_from_node		=	"Node functionality"
	current_node_name		string
)

func init() {
	tree_event.ON(tree_event.ON_RESTART_NODE, func(ev *tree_event.Event) {
		Restart()
	})
}
func SetParent(name string) bool {
	var err tree_lib.TreeError
	err.From = tree_lib.FROM_SET_PARENT
	node_info.ParentNodeInfo, err = tree_db.GetNodeInfo(name)
	if !err.IsNull() {
		tree_log.Error(err.From, "Getting parent node info from Node database, ", err.Error())
		return false
	}
	return true
}

func SetCurrentNode(name string) (err tree_lib.TreeError) {
	err.From = tree_lib.FROM_SET_CURRENT_NODE
	err = tree_db.Set(tree_db.DB_RANDOM, []byte("current_node"), []byte(name))
	return
}

func node_init() {
	// Getting current node name
	current_node_byte, err := tree_db.Get(tree_db.DB_RANDOM, []byte("current_node"))
	err.From = tree_lib.FROM_NODE_INIT
	if !err.IsNull() {
		tree_log.Error(err.From, "Getting current node name from Random database, ", err.Error())
		return
	}
	current_node_name = string(current_node_byte)
	node_info.CurrentNodeInfo, err = tree_db.GetNodeInfo(current_node_name)
	if !err.IsNull() {
		tree_log.Error(err.From, "Getting current node info from Node database, ", err.Error())
		return
	}

	// Setting current Node Value field from string to big.Int
	node_info.CurrentNodeValue = nil // Setting to nil for garbage collection
	node_info.CurrentNodeValue = big.NewInt(0)
	node_info.CurrentNodeValue.SetBytes(current_node_byte)

	for _, child :=range node_info.CurrentNodeInfo.Childs {
		node_info.ChildsNodeInfo[child], err = tree_db.GetNodeInfo(child)
		if !err.IsNull() {
			tree_log.Error(err.From, fmt.Sprintf("Getting child (%s) node info from Node database, ", child), err.Error())
			return
		}
	}

	// Setting relations
	tree_db.SetRelations(current_node_name)

	node_info.ParentNodeInfo, err = tree_db.GetParentInfo(current_node_name)
	if !err.IsNull() {
		tree_log.Error(err.From, "Getting parent node info from Node database, ", err.Error())
		return
	}

	// Setting node values based on child list
	node_info.CalculateChildParentNodeValues()
}

func Start() {
	node_init()
	InitBalancers()
	InitContainerClient()
	tree_net.Start()
	return
}

func Restart() {
	node_init()
	tree_net.Restart()
}

func InitBalancers() {
	err := tree_db.ForEach(tree_db.DB_BALANCER, func(k []byte, v []byte)error{
		var err  error
		bc := tree_balancer.BalancerConfig{Name: string(k)}
		err = ffjson.Unmarshal(v, &bc)
		if err != nil {
			return err
		}
		// TODO: Maybe we need to collect balancer services in MAP or Array
		_, err = tree_balancer.NewBalancerFromConfig(bc)
		return err
	})
	err.From = tree_lib.FROM_INIT_BALANCER
	if !err.IsNull() {
		tree_log.Error(err.From, "Unable to Init Balancers", err.Error())
		return
	}
}

func InitContainerClient() {
	go func() {
		for {
			// When Docker Client will be exited it will try to init again every 2 seconds
			tree_docker.StartEventListener()
			time.Sleep(time.Second * 2)
		}
	}()
}