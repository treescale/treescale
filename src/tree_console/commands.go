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

	build_tree := &cobra.Command{
		Use: "build [options]",
		Short: "Install nesessarry software and run TreeScale daemon for building relations",
		Long: `This command installs Docker, TreeScale and Netlink3 (for TreeScale networking).
			Need's to have SSH accesses or just access to current machine SSH Agent without providing SSH keys or passwords`,
		Run: BuildCmdHandler,
	}
	build_tree.Flags().BoolP("silent", "s", false, "If this flag persent, then on every 'install or not ?' question would be automatically answered 'Yes'")
	build_tree.Flags().BoolP("force", "f", false, "This flag forces installed software and reinstalling it again")
	build_tree.Flags().StringP("type", "t", "toml", "Configuration file format [toml, json, yaml] default is TOML")
	build_tree.Flags().StringSliceP("path", "p", []string{"."}, "Give a Path to directories containing configuration files")
	build_tree.Flags().StringSlice("files", []string{}, "Give file path list of configuration files")

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
		compile_config.Flags().StringP("type", "t", "toml", "Configuration file format [toml, json, yaml] default is TOML")
		compile_config.Flags().StringSliceP("path", "p", []string{"."}, "Give a Path to directory of configuration files")
		compile_config.Flags().StringSliceP("files", "f", []string{"console.toml", "treescale.toml"}, "Give file path list of configuration files")
		compile_config.Flags().StringP("out", "o", "tree.db", "Output file for compiled config files")

		restore_db := &cobra.Command{
			Use: "restore [options]",
			Short: "Restore compiled config to local database",
			Long: "This command restoring compiled and dumped file to local database for running TreeScale as a daemon",
			Run: RestoreFromConfigDump,
		}
		restore_db.Flags().StringP("file", "f", "tree.db", "Path to dumped file")

	config.AddCommand(compile_config, restore_db)


	// Node console commands
	node_cmd := &cobra.Command{
		Use: "node [options]",
		Short: "Command for running node as a Tree component",
		Long: `This command starting Tree Networking and Event handling
		Note: before running this command make sure this node have Tree Dabase restored, or transfered
		`,
		Run: HandleNodeCommand,
	}
	node_cmd.Flags().BoolP("daemon", "d", false, "Run Node in daemon mode")
	node_cmd.Flags().StringP("name", "n", "", "Set Node name for running it (needs to set for the first time and if it needs to be changed)")
	node_cmd.Flags().String("set-name", "", "Set Node name as a current node name in database")


	// API commands
	api_cmd := &cobra.Command{
		Use: "api [commands]",
		Short: "Send API commands to nodes and get results",
	}
		api_cmd_exec := &cobra.Command{
			Use: "exec [options]",
			Short: "Execute shell commands on specific Nodes",
			Run: HandleApiExec,
		}
		add_api_default_flags(api_cmd_exec)
		api_cmd_exec.Flags().StringP("cmd", "c", "uname", "Shell command to execute")
	api_cmd.AddCommand(api_cmd_exec)


	TreeScaleCMD.AddCommand(version, build_tree, config, node_cmd, api_cmd)
}

// Adding default flags for all API commands or related to that
func add_api_default_flags(cmd *cobra.Command)  {
	cmd.Flags().StringSliceP("node", "n", []string{""}, "Node name which will be API worker")
	cmd.Flags().StringSliceP("target", "t", []string{""}, "List of Node Names which will be as a target nodes for sending command")
	cmd.Flags().StringSlice("group", []string{}, "List of Node Groups for sending specific command for all")
	cmd.Flags().StringSlice("tag", []string{}, "List of Node Tags for sending specific command for all")
}