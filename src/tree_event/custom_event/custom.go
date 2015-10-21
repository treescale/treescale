package custom_event
import (
	"tree_db"
	"tree_lib"
	"github.com/pquerna/ffjson/ffjson"
	"fmt"
	"os/exec"
	"io"
)

type Handler struct {
	// Script or command line string which will be executed on specific event trigger
	Handler		string		`json:"handler" toml:"handler" yaml:"handler"`
	// If true then handler is a path to executable file, else it is command line string
	IsFile		bool		`json:"is_file" toml:"is_file" yaml:"is_file"`
	// Username who will be executing this handler on current linux os
	ExecUser	string		`json:"user" toml:"user" yaml:"user"`
	Password	string		`json:"password" toml:"password" yaml:"password"`

	// Reserved variable for putting some useful data in handler
	Data 		[]byte		`json:"user" toml:"user" yaml:"user"`
}

// Adding new custom event handler
// if event doesn't exists it will be created
func ON(name string, handlers...Handler) (err tree_lib.TreeError) {
	var (
		handlers_data	[]byte
		event_byte 		[]byte
		old_handlers	[]Handler
	)
	event_byte, err = tree_db.Get(tree_db.DB_EVENT, []byte(name))
	if !err.IsNull() {
		return
	}
	if len(event_byte) > 0 {
		err.Err = ffjson.Unmarshal(event_byte, &old_handlers)
		if !err.IsNull() {
			return
		}
	}

	old_handlers = append(old_handlers, handlers...)
	handlers_data, err.Err = ffjson.Marshal(old_handlers)
	err = tree_db.Set(tree_db.DB_EVENT, []byte(name), handlers_data)
	return
}

// Trigger specific event
func Trigger(name string, out io.Writer) (err tree_lib.TreeError) {
	var (
		event_byte 		[]byte
		handlers		[]Handler
	)
	event_byte, err = tree_db.Get(tree_db.DB_EVENT, []byte(name))
	if !err.IsNull() {
		return
	}
	err.Err = ffjson.Unmarshal(event_byte, &handlers)
	if !err.IsNull() {
		return
	}

	// calling handlers for this event
	for _, h :=range handlers {
		ExecuteHandler(h, out)
	}

	return
}

// Function for executing handlers as an OS command using exec package
func ExecuteHandler(handler Handler, out io.Writer) {
	var (
		cmd			*exec.Cmd
		command		string
		cmd_args	[]string
	)

	command = "/bin/sh"

	if handler.IsFile {
		cmd_args = []string{handler.Handler}
	} else {
		cmd_args = []string{"-c", fmt.Sprintf(`'%s'`, handler.Handler)}
	}

	if len(handler.ExecUser) > 0 {
		if len(handler.Password) > 0 {
			command = "echo"
			cmd_args = append([]string{handler.Password, "|", "sudo", "-S", "-u", handler.ExecUser}, cmd_args...)
		} else {
			command = "sudo"
			cmd_args = append([]string{"-S", "-u", handler.ExecUser}, cmd_args...)
		}
	}

	cmd = exec.Command(command, cmd_args...)
	cmd.Stdout = out
	cmd.Stderr = out
	cmd.Run()
}