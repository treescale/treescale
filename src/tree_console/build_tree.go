package tree_console

import (
	"fmt"
	"os"
	"bufio"
	"strings"
	"reflect"
	"golang.org/x/crypto/ssh"
	"bytes"
	"time"
	"tree_db"
	"github.com/spf13/cobra"
	"tree_lib"
	"tree_event"
	"tree_log"
	"sync"
)

const (
	log_from_tree_build		=	"Tree Build"
)

var (
	tmp_db_dir	string
)

func BuildCmdHandler(cmd *cobra.Command, args []string) {
	var (
		silent, force, multiple	bool
		err 					error
	)

	silent, err = cmd.Flags().GetBool("silent")
	if err != nil {
		tree_log.Error(log_from_tree_build, "Unable to get 'silent' flag", err.Error())
		return
	}

	force, err = cmd.Flags().GetBool("force")
	if err != nil {
		tree_log.Error(log_from_tree_build, "Unable to get 'force' flag", err.Error())
		return
	}

	multiple, err = cmd.Flags().GetBool("multiple")
	if err != nil {
		tree_log.Error(log_from_tree_build, "Unable to get 'force' flag", err.Error())
		return
	}

	generate_tmp_db_dir()
	// Adding fake flag for not duplicating code, and calling 'config' command
	cmd.Flags().String("out", tmp_db_dir, "")
	fmt.Println("Dumping Database file for transfer -> ", tmp_db_dir)
	CompileConfig(cmd, args) // After this step we have config in GLOBAL_CONFIG  variable and db file for sending to nodes

	BuildTree(&GLOBAL_CONFIG, silent, force, multiple)
}


func generate_tmp_db_dir() {
	tmp_db_dir = fmt.Sprintf("/tmp/%s", tree_lib.RandomFileName(15))
	// Adding event on program exited or terminated, it will automatically remove out tmp database file
	tree_event.ON(tree_event.ON_PROGRAM_EXIT, func(e *tree_event.Event){
		os.Remove(tmp_db_dir)
	})
}

func runSudo(pass, cmd string) string {
	return fmt.Sprintf("echo %s | sudo -S /bin/sh -c '%s'", pass, cmd)
}

func BuildTree(console_conf *TreeScaleConf, silent_build, force, multiple bool) {
	var (
		err			tree_lib.TreeError
		db_dump	=	tmp_db_dir
		wg		=	sync.WaitGroup{}
	)

	var BuildFunc = func(name string, ssh_conf SSHConfig) {
		var (
			input 			= 	make(chan string)
			test_err			*ssh.ExitError
		)

		fmt.Println("Connecting to Server -> ", name)
		err = ssh_conf.Connect()
		if !err.IsNull() {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}

		fmt.Println(name, " -> ", "Checking Docker availability")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "docker -v"), os.Stdout, os.Stderr, input)
		if !err.IsNull() {
			if reflect.TypeOf(err.Err).AssignableTo(reflect.TypeOf(test_err)) {
				fmt.Println(name, " -> ", "Docker is not Installed, Do you want to install it ? [Y/N]")
				if  silent_build || consoleYesNoWait() {
					err = installDocker(ssh_conf)
					if !err.IsNull() {
						fmt.Println(err.Error())
						fmt.Println("Terminating...")
						return
					}
				} else {
					fmt.Println(name, " -> ", "Docker installation skipped")
				}
			} else {
				fmt.Println(err.Error())
				fmt.Println("Terminating...")
				return
			}
		}

		fmt.Println(name, " -> ", "Checking TreeScale availability")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "treescale v"), os.Stdout, os.Stderr, input)
		if force || !err.IsNull() {
			if force || reflect.TypeOf(err.Err).AssignableTo(reflect.TypeOf(test_err)) {
				fmt.Println(name, " -> ", "TreeScale is not Installed, Do you want to install it ? [Y/N]")
				if silent_build || consoleYesNoWait() {
					err = installTreeScale(ssh_conf)
					if !err.IsNull() {
						fmt.Println(err.Error())
						fmt.Println("Terminating...")
						return
					}
				} else {
					fmt.Println("Infrastructure Tree will work only with TreeScale software !")
					fmt.Println("Terminating...")
					return
				}
			} else {
				fmt.Println(err.Error())
				fmt.Println("Terminating...")
			}
		}

		fmt.Println(name, " -> ", "Getting home directory for user ", ssh_conf.Username)
		home_dir_buf := bytes.NewBuffer([]byte{})
		err = ssh_conf.Exec("echo $HOME", home_dir_buf, os.Stderr, input)
		if !err.IsNull() {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}
		db_copy_path := fmt.Sprintf("%s/tree.db", strings.Replace(home_dir_buf.String(), "\n", "", -1))
		fmt.Println(name, " -> ", "Copeing Database dump file ", db_copy_path)
		err = ssh_conf.CopyFile(db_dump, db_copy_path)
		if !err.IsNull() {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}

		fmt.Println(name, " -> ", "Moving remote file to ", tree_db.DB_DIR)
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, fmt.Sprintf("mv %s %s", db_copy_path, tree_db.DEFAULT_DB_FILE)), os.Stdout, os.Stderr, input)
		if !err.IsNull() {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}

		fmt.Println(name, " -> ", "Setting name for current node")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, fmt.Sprintf("treescale node --set-name=%s", name)), os.Stdout, os.Stderr, input)
		if !err.IsNull() {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}

		fmt.Println("Adding private registry SSL exception")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "treescale ssl-exception"), os.Stdout, os.Stderr, input)
		if !err.IsNull() {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}
		fmt.Println("Restarting Docker")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "service docker restart"), os.Stdout, os.Stderr, input)
		if !err.IsNull() {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}
		time.Sleep(time.Second * 1)
		fmt.Println(name, " -> ", "Checking Docker Registery Server !")

		for _, reg :=range console_conf.Registry {
			if name == reg.Server {
				fmt.Println(name, " -> ", "Installing Docker registery")
				err = ssh_conf.Exec(runSudo(ssh_conf.Password, "docker pull registry:2"), os.Stdout, os.Stderr, input)
				if !err.IsNull() {
					fmt.Println(err.Error())
					fmt.Println("Terminating...")
					return
				}

				fmt.Println(fmt.Sprintf("Running Registery container on Port %d", reg.Port))
				err = ssh_conf.Exec(runSudo(ssh_conf.Password, fmt.Sprintf("docker run -d -p %d:%d registry:2", reg.Port, reg.Port)), os.Stdout, os.Stderr, input)
				if !err.IsNull() {
					fmt.Println(err.Error())
					fmt.Println("Terminating...")
					return
				}

				fmt.Println(fmt.Sprintf("Docker registery runned successfully on IP: %s and Prot: %d", reg.Server, reg.Port))
			}
		}

		fmt.Println(name, " -> ", "Running TreeScale in daemon mode")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "pkill treescale"), os.Stdout, os.Stderr, input)
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "treescale node -d"), os.Stdout, os.Stderr, input)
		if !err.IsNull() {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}

		fmt.Println(name, " -> ", "Tree component ready")
		ssh_conf.Disconnect()
		if multiple {
			wg.Done()
		}
	}


	err.From = tree_lib.FROM_BUILD_TREE
	for name, ssh_conf :=range console_conf.SSH {
		if multiple {
			wg.Add(1)
			go BuildFunc(name, ssh_conf)
		} else {
			BuildFunc(name, ssh_conf)
		}
	}
	if multiple {
		wg.Wait()
	}
}

func installDocker(ssh_conf SSHConfig) (err tree_lib.TreeError) {
	var (
		input 	= 	make(chan string)
		cmd			string
	)
	err.From = tree_lib.FROM_INSTALL_DOCKER
	cmd = `
	((apt-get update || true && apt-get install -y curl || true); yum install -y curl || true ) && curl -sSL https://get.docker.com/ | sudo sh
	`

	err = ssh_conf.Exec(runSudo(ssh_conf.Password, cmd), os.Stdout, os.Stderr, input)
	return
}


func installTreeScale(ssh_conf SSHConfig) (err tree_lib.TreeError) {
	var (
		input 	= 	make(chan string)
		cmd			string
	)
	err.From = tree_lib.FROM_INSTALL_TREESCALE
	cmd = `
	((apt-get update || true && apt-get install -y curl || true); yum install -y curl || true ) && curl -sSL https://source.treescale.com/install | sudo sh
	`

	err = ssh_conf.Exec(runSudo(ssh_conf.Password, cmd), os.Stdout, os.Stderr, input)
	return
}


func consoleYesNoWait() (yes bool) {
	yes = false
	for {
		reader := bufio.NewReader(os.Stdin)
		text, _ := reader.ReadString('\n')
		text = strings.Replace(text, "\n", "", -1)
		text = strings.Replace(text, "\r", "", -1)
		switch strings.ToLower(text) {
		case "y", "yes":
			{
				yes = true
				return
			}
		case "n", "no":
			{
				yes = false
				return
			}
		default:
			{
				fmt.Println("Please type (y, Y, yes, YES) or (n, N, no, NO) : ")
				continue
			}
		}
	}
	return
}