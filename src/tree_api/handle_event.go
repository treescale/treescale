package tree_api
import (
	"tree_event"
	"tree_event/custom_event"
	"tree_lib"
	"tree_log"
	"github.com/pquerna/ffjson/ffjson"
)

func HandleTriggerCustomEvent(e *tree_event.Event, api_cmd Command) {
	var (
		out			=	&WriterCallback{BufferMaxSize: 1024}
		event_name	=	string(api_cmd.Data)
		err 			tree_lib.TreeError
		ev_data			[]byte
	)
	err.From = tree_lib.FROM_TRIGGER_CUSTOM_EVENT
	out.OutCallback = func(data []byte, ended bool) {
		cb_cmd := api_cmd
		cb_cmd.Ended = ended
		cb_cmd.Data = data
		ev_data, err.Err = ffjson.Marshal(cb_cmd)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return
		}
		SendCommandCallback(e, ev_data)
	}
	defer out.End()
	err = custom_event.Trigger(event_name, out)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
	}
}

func HandleAddCustomEventHandlers(e *tree_event.Event, api_cmd Command)  {
	var (
		out			=	&WriterCallback{BufferMaxSize: 1024}
		handler_data	map[string]interface{}
		err 			tree_lib.TreeError
		ev_data			[]byte
	)
	err.From = tree_lib.FROM_ADD_CUSTOM_EVENT
	err.Err = ffjson.Unmarshal(api_cmd.Data, &handler_data)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	out.OutCallback = func(data []byte, ended bool) {
		cb_cmd := api_cmd
		cb_cmd.Ended = ended
		cb_cmd.Data = data
		ev_data, err.Err = ffjson.Marshal(cb_cmd)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return
		}
		SendCommandCallback(e, ev_data)
	}
	defer out.End()

	event_name := handler_data["name"].(string)
	handles_interfaces := handler_data["handlers"].([]interface{})
	var (
		handles_interfaces_data []byte
		handlers	[]custom_event.Handler
	)
	handles_interfaces_data, err.Err = ffjson.Marshal(handles_interfaces)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	err.Err = ffjson.Unmarshal(handles_interfaces_data, &handlers)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	err = custom_event.ON(event_name, handlers...)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
	}
}