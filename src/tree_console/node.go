package tree_console
import (
	"github.com/spf13/cobra"
	"tree_node"
	"tree_log"
	"fmt"
	"tree_db"
	"tree_event"
	"time"
	"os/exec"
	"os"
	"log"
	"tree_lib"
)

const (
	log_from_node_console = "Console functionality for Node"
)

func HandleNodeCommand(cmd *cobra.Command, args []string) {
	var (
		name		string
		err 		tree_lib.TreeError
	)
	err.From = tree_lib.FROM_HANDLE_NODE_COMMAND
	name, err.Err = cmd.Flags().GetString("set-name")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
	}

	// If we have set-name flag then we just setting current_node in database and exiting
	if len(name) > 0 {
		tree_db.Set(tree_db.DB_RANDOM, []byte("current_node"), []byte(name))
		return
	}
	daemon := false
	daemon, err.Err = cmd.Flags().GetBool("daemon")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	if daemon {
		cmd := exec.Command("/bin/sh", "-c", fmt.Sprintf("%s node > %s 2>&1 &", os.Args[0], tree_log.LogFile))
		err.Err = cmd.Run()
		if !err.IsNull() {
			log.Fatal(err.Err)
		}
		return
	}

	name, err.Err = cmd.Flags().GetString("name")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	if len(name) == 0 {
		current_node_byte, err := tree_db.Get(tree_db.DB_RANDOM, []byte("current_node"))
		if !err.IsNull() {
			tree_log.Error(err.From, "Getting current node name from Random database, ", err.Error())
			return
		}
		if len(current_node_byte) == 0 {
			fmt.Println("Name is important for the first time run")
			return
		}
	} else {
		err = tree_node.SetCurrentNode(name)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return
		}
	}

	tree_event.ON("test", func(e *tree_event.Event) {
		fmt.Println(e.Data)
	})

	go func() {
		var err tree_lib.TreeError
		err.From = tree_lib.FROM_HANDLE_NODE_COMMAND
		time.Sleep(time.Second * 2)
		if name == "tree1" {
			em := &tree_event.EventEmitter{}
			em.Name = "test"
			em.Data = []byte("aaaaaaaaaaaaaaaa")
			em.ToNodes = []string{"tree2"}
			err = tree_event.Emit(em)
			if !err.IsNull() {
				fmt.Println(err.Error())
			}
		}
	}()

	tree_node.Start()
}