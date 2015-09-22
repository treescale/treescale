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
)


func runSudo(pass, cmd string) string {
	return fmt.Sprintf("echo %s | sudo -S /bin/sh -c '%s'", pass, cmd)
}

func BuildTree(console_conf *TreeScaleConf, silent_build, force bool) {
	var (
		err			error
		db_dump	=	"tree.db"
	)

	fmt.Println("Dumping Database file for transfer -> ", db_dump)
	tree_db.DumpDBPath(db_dump)

	for name, ssh_conf :=range console_conf.SSH {
		var (
			input 			= 	make(chan string)
			test_err			*ssh.ExitError
		)

		fmt.Println("Connecting to Server -> ", name)
		err = ssh_conf.Connect()
		if err != nil {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}

		fmt.Println(name, " -> ", "Checking Docker availability")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "docker -v"), os.Stdout, os.Stderr, input)
		if err != nil {
			if reflect.TypeOf(err).AssignableTo(reflect.TypeOf(test_err)) {
				fmt.Println(name, " -> ", "Docker is not Installed, Do you want to install it ? [Y/N]")
				if  silent_build || consoleYesNoWait() {
					err = installDocker(ssh_conf)
					if err != nil {
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
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "treescale -v"), os.Stdout, os.Stderr, input)
		if force || err != nil {
			if force || reflect.TypeOf(err).AssignableTo(reflect.TypeOf(test_err)) {
				fmt.Println(name, " -> ", "TreeScale is not Installed, Do you want to install it ? [Y/N]")
				if silent_build || consoleYesNoWait() {
					err = installTreeScale(ssh_conf)
					if err != nil {
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
		if err != nil {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}
		fmt.Println(name, " -> ", "Copeing Database dump file ", fmt.Sprintf("%s/tree.db", strings.Replace(home_dir_buf.String(), "\n", "", -1)))
		err = ssh_conf.CopyFile(db_dump, fmt.Sprintf("%s/tree.db", strings.Replace(home_dir_buf.String(), "\n", "", -1)))
		if err != nil {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}

		fmt.Println(name, " -> ", "Moving remote file to ", tree_db.DB_DIR)
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, fmt.Sprintf("mv $HOME/tree.db %s", tree_db.DB_DIR)), os.Stdout, os.Stderr, input)
		if err != nil {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}

		fmt.Println(name, " -> ", "Setting name for current node")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, fmt.Sprintf("treescale node --set-name=%s", name)), os.Stdout, os.Stderr, input)
		if err != nil {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}

		fmt.Println("Adding private registry SSL exception")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "treescale ssl-exception"), os.Stdout, os.Stderr, input)
		if err != nil {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}
		fmt.Println("Restarting Docker")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "service docker restart"), os.Stdout, os.Stderr, input)
		if err != nil {
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
				if err != nil {
					fmt.Println(err.Error())
					fmt.Println("Terminating...")
					return
				}

				fmt.Println(fmt.Sprintf("Running Registery container on Port %d", reg.Port))
				err = ssh_conf.Exec(runSudo(ssh_conf.Password, fmt.Sprintf("docker run -d -p %d:%d registry:2", reg.Port, reg.Port)), os.Stdout, os.Stderr, input)
				if err != nil {
					fmt.Println(err.Error())
					fmt.Println("Terminating...")
					return
				}

				fmt.Println(fmt.Sprintf("Docker registery runned successfully on IP: %s and Prot: %d", reg.Server, reg.Port))
			}
		}

		fmt.Println(name, " -> ", "Running TreeScale in daemon mode")
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "killall treescale"), os.Stdout, os.Stderr, input)
		err = ssh_conf.Exec(runSudo(ssh_conf.Password, "treescale -d"), os.Stdout, os.Stderr, input)
		if err != nil {
			fmt.Println(err.Error())
			fmt.Println("Terminating...")
			return
		}

		fmt.Println(name, " -> ", "Tree component ready")
		ssh_conf.Disconnect()
	}
}

func installDocker(ssh_conf SSHConfig) (err error) {
	var (
		input 	= 	make(chan string)
		cmd			string
	)

	cmd = `
	apt-get update && sudo apt-get install -y curl && curl -sSL https://get.docker.com/ | sudo sh
	`

	err = ssh_conf.Exec(runSudo(ssh_conf.Password, cmd), os.Stdout, os.Stderr, input)
	return
}


func installTreeScale(ssh_conf SSHConfig) (err error) {
	var (
		input 	= 	make(chan string)
		cmd			string
	)

	cmd = `
	apt-get update && \
	sudo apt-get install -y curl && \
	curl -sSL https://console.treescale.com/install | sudo sh
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