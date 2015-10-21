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
			Use: "exec",
			Short: "Execute shell commands on specific Nodes",
			Run: HandleApiExec,
		}
		add_api_default_flags(api_cmd_exec)
		api_cmd_exec.Flags().StringP("cmd", "c", "uname", "Shell command to execute")
	api_cmd.AddCommand(api_cmd_exec)

	info_cmd := &cobra.Command{
		Use: "info [commands]",
		Short: "Update or Get database info",
	}
		info_cmd_list := &cobra.Command{
			Use: "list",
			Short: "Listing node infos",
			Run: ListInfos,
		}
		add_list_default_flags(info_cmd_list)
		info_cmd_update := &cobra.Command{
			Use: "update",
			Short: "Update node infos",
			Run: UpdateInfo,
		}
		add_update_default_flags(info_cmd_update)
	info_cmd.AddCommand(info_cmd_list, info_cmd_update)
	container_cmd := &cobra.Command{
		Use: "container [commands]",
		Short: "Manage docker containers",
	}
		container_start_cmd := &cobra.Command{
			Use: "start",
			Short: "Start docker container",
			Run: HandleContStart,
		}
		add_container_flags(container_start_cmd)
		add_start_flags(container_start_cmd)
		container_stop_cmd := &cobra.Command{
			Use: "stop",
			Short: "Stop docker container",
			Run: HandleStop,
		}
		add_container_flags(container_stop_cmd)
		add_stop_flags(container_stop_cmd)
		container_create_cmd := &cobra.Command{
			Use: "create",
			Short: "Create container",
			Run: HandleCreate,
		}
		add_container_flags(container_create_cmd)
		add_start_flags(container_create_cmd)
		add_create_flags(container_create_cmd)
		container_pause_cmd := &cobra.Command{
			Use: "pause",
			Short: "Pause docker container",
			Run: HandlePause,
		}
		add_container_flags(container_pause_cmd)
		container_resume_cmd := &cobra.Command{
			Use: "resume",
			Short: "Resume docker container",
			Run: HandleResume,
		}
		add_container_flags(container_resume_cmd)
		container_delete_cmd := &cobra.Command{
			Use: "delete",
			Short: "Delete docker container",
			Run: HandleDelete,
		}
		add_container_flags(container_delete_cmd)
		container_inspect_cmd := &cobra.Command{
			Use: "inspect",
			Short: "Inspect docker container",
			Run: HandleInspect,
		}
		add_container_flags(container_inspect_cmd)
		container_list_cmd := &cobra.Command{
			Use: "list",
			Short: "list docker containers",
			Run: HandleList,
		}
		add_list_flags(container_list_cmd)
		image_cmd := &cobra.Command{
			Use: "image [commands]",
			Short: "manage images",
		}
			image_list_cmd := &cobra.Command{
				Use: "list",
				Short: "list docker images",
				Run: HandleList,
			}
			add_list_flags(image_list_cmd)
			image_delete_cmd := &cobra.Command{
				Use: "delete",
				Short: "delete image",
				Run: HandleImageDelete,
			}
			add_image_delete_flags(image_delete_cmd)
			image_pull_cmd := &cobra.Command{
				Use: "pull",
				Short: "pull image",
				Run: HandleImagePull,
			}
			add_image_pull_flags(image_pull_cmd)
		image_cmd.AddCommand(image_list_cmd, image_delete_cmd, image_pull_cmd)
	container_cmd.AddCommand(container_start_cmd, container_stop_cmd, container_create_cmd, container_pause_cmd, container_resume_cmd, container_delete_cmd, container_inspect_cmd)
	container_cmd.AddCommand(container_list_cmd, image_cmd)

	// Custom Events
	event_cmd := &cobra.Command{
		Use: "event [commands]",
		Short: "Manage infrastructure events",
	}
		event_add_cmd := &cobra.Command{
			Use: "add [options]",
			Short: "Add new event handler or event if it's new event",
			Run: SendAddEventHandlerCommand,
		}
		add_api_default_flags(event_add_cmd)
		event_add_cmd.Flags().StringP("event", "e", "", "Event Name for adding it to Nodes")
		event_add_cmd.Flags().String("handler", "", "Command line string for handling this event trigger")
		event_add_cmd.Flags().Bool("file", false, "Is this a file")
		event_add_cmd.Flags().String("user", "", "Is this a file")
		event_add_cmd.Flags().String("pass", "", "Is this a file")
		event_trigger_cmd := &cobra.Command{
			Use: "trigger [options]",
			Short: "Trigger event for specific nodes",
			Run: SendEventTriggerCommand,
		}
		event_trigger_cmd.Flags().StringP("event", "e", "", "Event Name for adding it to Nodes")
		add_api_default_flags(event_trigger_cmd)

	event_cmd.AddCommand(event_add_cmd, event_trigger_cmd)

	TreeScaleCMD.AddCommand(version, build_tree, config, node_cmd, api_cmd, info_cmd, container_cmd, event_cmd)
}

// Adding default flags for all API commands or related to that
func add_api_default_flags(cmd *cobra.Command)  {
	cmd.Flags().StringP("node", "n", "", "Node name which will be API worker")
	cmd.Flags().StringSliceP("target", "t", []string{""}, "List of Node Names which will be as a target nodes for sending command")
	cmd.Flags().StringSlice("group", []string{}, "List of Node Groups for sending specific command for all")
	cmd.Flags().StringSlice("tag", []string{}, "List of Node Tags for sending specific command for all")
}

func add_list_default_flags(cmd *cobra.Command) {
	cmd.Flags().StringP("node", "n", "", "Node name which will be API worker")
	cmd.Flags().StringSliceP("target", "t", []string{""}, "Node names which infos wiil be listed")
	cmd.Flags().StringSlice("group", []string{""}, "Group names which node infos will be listed")
	cmd.Flags().StringSlice("tag", []string{""}, "Tag names which node infos will be listed")
}

func add_update_default_flags(cmd *cobra.Command) {
	cmd.Flags().StringP("node", "n", "", "Node name which will be API worker")
	cmd.Flags().StringP("target", "t", "", "Node name which info will be updated")
	cmd.Flags().String("add_child", "", "Node name which wiil be added as child")
	cmd.Flags().StringP("ip", "i", "", "new ip address of node")
	cmd.Flags().IntP("port", "p", -1, "new port of node")
	cmd.Flags().String("add_group", "", "Goup name whom node will be added")
	cmd.Flags().String("delete_child", "", "Child name which will be deleted")
	cmd.Flags().String("delete_group", "", "Group name from node will be deleted")
	cmd.Flags().String("add_tag", "", "Tag name whom node will be added")
	cmd.Flags().String("delete_tag", "", "tag name from node will fiil be deleted")
}
func add_container_flags(cmd *cobra.Command){
	cmd.Flags().StringP("node", "n", "", "Node name which will be API worker")
	cmd.Flags().StringSliceP("target", "t", []string{""}, "Node name where will be start/create/stop container")
	cmd.Flags().StringP("container", "c", "", "Container name")
}
func add_start_flags(cmd *cobra.Command) {
	cmd.Flags().StringP("image", "i", "", "Image name where will be start/create container")
	cmd.Flags().String("command", "", "Conatainer start Command")
	cmd.Flags().String("cpu", "", "CPU Shares")
	cmd.Flags().String("ram", "", "Ram")
}
func add_create_flags(cmd *cobra.Command) {
	cmd.Flags().BoolP("start", "s", false, "Start container or not")
	cmd.Flags().String("count", "", "How many container create")
}
func add_stop_flags(cmd *cobra.Command) {
	cmd.Flags().String("time", "", "Seconds to wait for stop before killing it")
}

func add_list_flags(cmd *cobra.Command){
	cmd.Flags().StringP("node", "n", "", "Node name which will be API worker")
	cmd.Flags().StringSliceP("target", "t", []string{""}, "Node names whose images will be listed")
}

func add_image_delete_flags(cmd *cobra.Command){
	cmd.Flags().StringP("node", "n", "", "Node name which will be API worker")
	cmd.Flags().StringSliceP("target", "t", []string{""}, "Node name where was the image")
	cmd.Flags().StringP("image", "i", "", "image name")
	cmd.Flags().BoolP("force", "f", false, "Force removal of the image")
}

func add_image_pull_flags(cmd *cobra.Command){
	cmd.Flags().StringP("node", "n", "", "Node name which will be API worker")
	cmd.Flags().StringSliceP("target", "t", []string{""}, "Node name where you want to pull image")
	cmd.Flags().StringP("registry", "r", "", "registry name")
	cmd.Flags().StringP("image", "i", "", "image name")
	cmd.Flags().StringP("username", "u", "", "registry username")
	cmd.Flags().StringP("password", "p", "", "registry password")
	cmd.Flags().StringP("email", "e", "", "registry email")
	cmd.Flags().StringP("address", "a", "", "registry address")
}