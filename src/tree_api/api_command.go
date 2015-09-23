package tree_api
import (
	"bytes"
	"tree_event"
	"tree_lib"
	"github.com/pquerna/ffjson/ffjson"
	"tree_log"
	tree_path "tree_graph/path"
)

const (
	API_OUTPUT_BUFFER_SIZE = 1024
	log_from_api_command = "API command functionality"
	// Command Types
	COMMAND_EXEC	=	0
)


type cb_str struct {
	f	func(ev *tree_event.Event, cmd Command)bool
	c	chan bool
}

var (
	subscribed_command_callbacks	=	make(map[string]cb_str)
)

func init() {
	// This event will be triggered from Node, when API client will send some command to implement
	tree_event.ON(tree_event.ON_API_COMMAND, func(ev *tree_event.Event){
		var err tree_lib.TreeError;
		err.From = tree_lib.FROM_INIT
		cmd := Command{}
		err.Err = ffjson.Unmarshal(ev.Data, &cmd)
		if  !err.IsNull() {
			tree_log.Error(err.From, "unable to unmarshal event data as a command -> ", err.Error())
			return
		}

		switch cmd.CommandType {
		case COMMAND_EXEC:
			{
				HandleExecCommand(ev, cmd)
			}
		}
	})

	// This event will be triggered from API client when Node will give callback for specific commands
	tree_event.ON(tree_event.ON_API_COMMAND_CALLBACK, func(ev *tree_event.Event){
		var err tree_lib.TreeError
		err.From = tree_lib.FROM_INIT
		cmd := Command{}
		err.Err = ffjson.Unmarshal(ev.Data, &cmd)
		if !err.IsNull(){
			tree_log.Error(err.From, "unable to unmarshal event data as a command -> ", err.Error())
			return
		}

		if cb, ok :=subscribed_command_callbacks[cmd.ID]; ok && cb.f != nil {
			if !cb.f(ev, cmd) {
				// TODO: Maybe we need mutex to lock deleting process
				delete(subscribed_command_callbacks, cmd.ID)
				if cb.c != nil {
					cb.c <- true	// Ending wait chanel in send command
				}
			}
		}
	})
}

type Command struct {
	ID				string					`json:"id" toml:"id" yaml:"id"`
	Data			[]byte					`json:"data" toml:"data" yaml:"data"`
	Ended			bool					`json:"ended" toml:"ended" yaml:"ended"`
	CommandType		int						`json:"command_type" toml:"command_type" yaml:"command_type"`
}

type WriterCallback struct {
	BufferMaxSize		int						`json:"buffer_max_size" toml:"buffer_max_size" yaml:"buffer_max_size"`
	OutCallback			func([]byte, bool)		`json:"-" toml:"-" yaml:"-"`	// function for getting callback data from command and is ended or not
	out_data			bytes.Buffer			`json:"-" toml:"-" yaml:"-"`
}

func (cb *WriterCallback) Write(p []byte) (n int, err error) {
	n, err = cb.out_data.Write(p)
	if err != nil {
		return
	}

	if cb.out_data.Len() >= cb.BufferMaxSize {
		cb.trigger_callback(false)
	}

	return
}

func (cb *WriterCallback) trigger_callback(ended bool) {
	go cb.OutCallback(cb.out_data.Bytes(), ended)
	cb.out_data.Reset()
}

func (cb *WriterCallback) End() {
	cb.trigger_callback(true)
}

func SendCommand(cmd *Command, targets []string, path *tree_path.Path, cb func(*tree_event.Event, Command)bool) (err tree_lib.TreeError) {
	// If command ID not set just setting random string
	if len(cmd.ID) == 0 {
		cmd.ID = tree_lib.RandomString(10)
	}

	var (
		cmd_data	[]byte
	)

	cmd_data, err.Err = ffjson.Marshal(cmd)
	if !err.IsNull() {
		err.From = tree_lib.FROM_SEND_COMMAND
		return
	}

	e := &tree_event.Event{
		Name: tree_event.ON_API_COMMAND,
		Data: cmd_data,
		Path: (*path),
	}

	EmitApi(e, targets...)

	if cb != nil {
		subscribed_command_callbacks[cmd.ID] = cb_str{f: cb, c: make(chan bool)}
		<- subscribed_command_callbacks[cmd.ID].c
	}

	return
}

