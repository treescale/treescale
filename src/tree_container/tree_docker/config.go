package tree_docker

import (
	"strings"
	"fmt"
	"os"
	"io/ioutil"
	"os/exec"
	"tree_lib"
)


type DockerRegistry struct {
	Server		string		`json:"server" toml:"server" yaml:"server"`
	IP			string		`json:"ip" toml:"ip" yaml:"ip"`
	Port		int			`json:"port" toml:"port" yaml:"port"`
	SSL			bool		`json:"ssl" toml:"ssl" yaml:"ssl"`  // Add SSL exception or not (adding exception if SSL is false - don't have SSL)
}


// Adding SSL exceptions for Docker private registry, because mostly it would be IP address or without any HTTPS
func (reg *DockerRegistry) SSLExceptions() (err tree_lib.TreeError) {
	var (
		f_data			[]byte
		lines			[]string
		ex_str 	= 		fmt.Sprintf(" --insecure-registry=%s:%d ", reg.IP, reg.Port)
		combine_str 	[]string
		filename		string
	)
	err.From = tree_lib.FROM_SSL_EXCEPTIONS
	// SystemD service
	filename = "/lib/systemd/system/docker.service"
	if _, err.Err = os.Stat(filename); !os.IsNotExist(err.Err) {
		f_data, err.Err = ioutil.ReadFile(filename)
		if !err.IsNull() {
			return
		}

		lines = strings.Split(string(f_data), "\n")
		for i, l :=range lines {
			if strings.Contains(l, "ExecStart") {
				str_split := strings.Split(l, " ")
				ex_contains := false
				for _, s :=range str_split {
					if strings.Contains(s, "--insecure-registry") {
						combine_str = append(combine_str, ex_str)
						ex_contains = true
						continue
					}
					if len(s) == 0 {
						continue
					}
					combine_str = append(combine_str, s)
				}

				if !ex_contains {
					combine_str = append(combine_str, ex_str)
				}

				lines[i] = strings.Join(combine_str, " ")
			}
		}

		err.Err = ioutil.WriteFile(filename, []byte(strings.Join(lines, "\n")), 0644)
		if !err.IsNull() {
			return
		}
	}


	// SystemD service
	filename = "/etc/default/docker"
	if _, err.Err = os.Stat(filename); !os.IsNotExist(err.Err) {
		f_data, err.Err = ioutil.ReadFile(filename)
		if !err.IsNull() {
			return
		}

		lines = strings.Split(string(f_data), "\n")
		for i, l :=range lines {
			tmp_sps := strings.Replace(l, " ", "", -1)
			if strings.Contains(tmp_sps, "DOCKER_OPTS=") {
				l = strings.Replace(l, "#", "", -1)
				l = strings.Replace(l, "\"", " ", -1)
				str_split := strings.Split(l, " ")
				ex_contains := false
				combine_str = make([]string, 0)
				for _, s :=range str_split {
					if strings.Contains(s, "--insecure-registry") {
						combine_str = append(combine_str, ex_str)
						ex_contains = true
						continue
					}
					if len(s) == 0 {
						continue
					}
					combine_str = append(combine_str, s)
				}

				if !ex_contains {
					combine_str = append(combine_str, ex_str)
				}

				lines[i] = strings.Join(combine_str, " ")
				lines[i] = strings.Replace(lines[i], "DOCKER_OPTS=", "DOCKER_OPTS=\"", -1)
				lines[i] = fmt.Sprintf("%s\"", lines[i])
			}
		}

		err.Err = ioutil.WriteFile(filename, []byte(strings.Join(lines, "\n")), 0644)
		if !err.IsNull() {
			return
		}
	}

	cmd := exec.Command("service", "docker", "restart")
	err.Err = cmd.Run()
	return
}