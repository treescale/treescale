package tree_console

import (
	"golang.org/x/crypto/ssh"
	"io/ioutil"
	"io"
	"bytes"
	"github.com/pkg/sftp"
	"fmt"
	"golang.org/x/crypto/ssh/agent"
	"os"
	"net"
	"tree_lib"
)


type SSHConfig struct {
	Host		string			`toml:"host" json:"host" yaml:"host"`
	Port		int				`toml:"port" json:"port" yaml:"port"`
	Username	string			`toml:"username" json:"username" yaml:"username"`
	Password	string			`toml:"password" json:"password" yaml:"password"`	// Password for ssh access
	conn		*ssh.Client		`toml:"-" json:"-" yaml:"-"`
}

func ssh_agent() ssh.AuthMethod {
	if sshAgent, err := net.Dial("unix", os.Getenv("SSH_AUTH_SOCK")); err == nil {
		return ssh.PublicKeysCallback(agent.NewClient(sshAgent).Signers)
	}
	return nil
}

func (ssc *SSHConfig) Connect() (err tree_lib.TreeError) {
	var (
		client_conf		ssh.ClientConfig
	)
	err.From = tree_lib.FROM_SSH_CONNECT
	client_conf.User = ssc.Username
	if len(ssc.Password) > 0 {
		client_conf.Auth = []ssh.AuthMethod{ssh.Password(ssc.Password)}
	} else {
		client_conf.Auth = []ssh.AuthMethod{ssh_agent()}
	}

	ssc.conn, err.Err = ssh.Dial("tcp", fmt.Sprintf("%s:%d", ssc.Host, ssc.Port), &client_conf)
	return
}

func (ssc *SSHConfig) Disconnect() (err tree_lib.TreeError) {
	err.From = tree_lib.FROM_SSH_DISCONNECT
	if ssc.conn != nil {
		err.Err = ssc.conn.Close()
	}
	return
}

func (ssc *SSHConfig) Exec(cmd string, stdout, stderr io.Writer, input chan string) (err tree_lib.TreeError) {
	var (
		session 		*ssh.Session
		stdin			io.WriteCloser
		command_ended	bool
	)
	err.From = tree_lib.FROM_SSH_EXEC
	session, err.Err = ssc.conn.NewSession()
	if !err.IsNull() {
		return
	}
	defer session.Close()

	session.Stdout = stdout
	session.Stderr = stderr

	stdin, err.Err = session.StdinPipe()
	if !err.IsNull() {
		return
	}

	err.Err = session.Start(cmd)
	if !err.IsNull() {
		return
	}
	command_ended = false
	go func () {
		for !command_ended {
			io.Copy(stdin, bytes.NewBufferString(<- input))
		}
	}()
	err.Err = session.Wait()
	command_ended = true
	return
}

func (ssc *SSHConfig) CopyFile(local_path, remote_path string) (err tree_lib.TreeError) {
	var (
		sft				*sftp.Client
		f				*sftp.File
		file_data		[]byte
	)
	err.From = tree_lib.FROM_SSH_COPY_FILE
	sft, err.Err = sftp.NewClient(ssc.conn)
	if !err.IsNull() {
		return
	}
	defer sft.Close()

	file_data, err.Err = ioutil.ReadFile(local_path)
	if !err.IsNull() {
		return
	}

	f, err.Err = sft.Create(remote_path)
	if !err.IsNull() {
		return
	}

	_, err.Err = f.Write(file_data)
	f.Close()
	sft.Close()
	return
}