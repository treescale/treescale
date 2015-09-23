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

func (ssc *SSHConfig) Connect() (err error) {
	var (
		client_conf		ssh.ClientConfig
	)

	client_conf.User = ssc.Username
	if len(ssc.Password) > 0 {
		client_conf.Auth = []ssh.AuthMethod{ssh.Password(ssc.Password)}
	} else {
		client_conf.Auth = []ssh.AuthMethod{ssh_agent()}
	}

	ssc.conn, err = ssh.Dial("tcp", fmt.Sprintf("%s:%d", ssc.Host, ssc.Port), &client_conf)
	return
}

func (ssc *SSHConfig) Disconnect() (err error) {
	if ssc.conn != nil {
		err = ssc.conn.Close()
	}
	return
}

func (ssc *SSHConfig) Exec(cmd string, stdout, stderr io.Writer, input chan string) (err error) {
	var (
		session 		*ssh.Session
		stdin			io.WriteCloser
		command_ended	bool
	)

	session, err = ssc.conn.NewSession()
	if err != nil {
		return
	}
	defer session.Close()

	session.Stdout = stdout
	session.Stderr = stderr

	stdin, err = session.StdinPipe()
	if err != nil {
		return
	}

	err = session.Start(cmd)
	if err != nil {
		return
	}
	command_ended = false
	go func () {
		for !command_ended {
			io.Copy(stdin, bytes.NewBufferString(<- input))
		}
	}()
	err = session.Wait()
	command_ended = true
	return
}

func (ssc *SSHConfig) CopyFile(local_path, remote_path string) (err error) {
	var (
		sft				*sftp.Client
		f				*sftp.File
		file_data		[]byte
	)

	sft, err = sftp.NewClient(ssc.conn)
	if err != nil {
		return
	}
	defer sft.Close()

	file_data, err = ioutil.ReadFile(local_path)
	if err != nil {
		return
	}

	f, err = sft.Create(remote_path)
	if err != nil {
		return
	}

	_, err = f.Write(file_data)
	f.Close()
	sft.Close()
	return
}