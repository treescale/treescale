package tree_console
import (
	"github.com/spf13/cobra"
	"fmt"
)

var (
	TreeScaleCMD	*cobra.Command
)

func init() {
	TreeScaleCMD = &cobra.Command{
		Use: 	"treescale [commands]",
		Short:	"Scaling and Resource Managaement evented system, based on Mathematical Tree and Graph",
		Long:	"",
	}

	// List of commands to execute
	version := &cobra.Command{
		Use: "version",
		Aliases: []string{"v"},
		Short: "Prints version of program",
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Println("TreeScale Version 0.001")
			fmt.Println("Copyright TreeScale Inc.")
		},
	}


	TreeScaleCMD.AddCommand(version)
}