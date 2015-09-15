package tree_console
import (
	"github.com/spf13/cobra"
	"fmt"
)

var (
	TreeScaleCMD	*cobra.Command
	KX	*[]string
)

func init() {
	TreeScaleCMD = &cobra.Command{
		Use: 	"treescale [commands]",
		Short:	"Scaling and Resource Managaement evented system, based on Mathematical Tree and Graph",
		Long:	"",
	}

	// Adding Flags


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

	// Configuration commands
	config := &cobra.Command{
		Use: 	"config [commands]",
		Short:	"Command for handling configuration tools",
	}
	compile_config := &cobra.Command{
		Use: "compile [options]",
		Short: "Compiles multiple config files",
		Long: "Compiles multiple config files into one single TreeScale database for sending to nodes",
		Run: CompileConfig,
	}
	compile_config.Flags().StringP("path", "p", ".", "Test Usage")
	compile_config.Flags().StringSliceP("pp", "l", []string{"."}, "Test Usage")
	config.AddCommand(compile_config)

	TreeScaleCMD.AddCommand(version, config)
}