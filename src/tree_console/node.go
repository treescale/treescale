package tree_console
import (
	"github.com/spf13/cobra"
	"tree_node"
	"tree_log"
	"fmt"
	"tree_db"
)

const (
	log_from_node_console = "Console functionality for Node"
)

func HandleNodeCommand(cmd *cobra.Command, args []string) {
	name, err := cmd.Flags().GetString("name")
	if err != nil {
		tree_log.Error(log_from_node_console, err.Error())
		return
	}

	if len(name) == 0 {
		current_node_byte, err := tree_db.Get(tree_db.DB_RANDOM, []byte("current_node"))
		if err != nil {
			tree_log.Error(log_from_node_console, "Getting current node name from Random database, ", err.Error())
			return
		}
		if len(current_node_byte) == 0 {
			fmt.Println("Name is important for the first time run")
			return
		}
	} else {
		err = tree_node.SetCurrentNode(name)
		if err != nil {
			tree_log.Error(log_from_node_console, err.Error())
			return
		}
	}

	tree_node.Start()
}