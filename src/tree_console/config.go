package tree_console
import (
	"tree_node/node_info"
	"github.com/spf13/cobra"
	"fmt"
)


type TreeScaleConf struct {
	SSH				map[string]SSHConfig				`toml:"ssh" json:"ssh" yaml:"ssh"`
	TreeNode		map[string]node_info.NodeInfo		`toml:"tree_node" json:"tree_node" yaml:"tree_node"`
	// TODO: Add docker registry config here
	// TODO: Add balancer config here
}

func CompileConfig(cmd *cobra.Command, args []string) {
	fmt.Println(cmd.Flag("pp").Value)
}