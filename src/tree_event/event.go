package tree_event

import (
	"tree_log"
	"github.com/pquerna/ffjson/ffjson"
	"reflect"
	"tree_lib"
	"tree_graph"
)


type Event struct {
	Name		string				`json:"name" toml:"name" yaml:"name"`
	Data		[]byte				`json:"data" toml:"data" yaml:"data"`
	From		string				`json:"from" toml:"from" yaml:"from"` // Name who sending this event
	FromApi		int64				`json:"from_api" toml:"from_api" yaml:"from_api"` // Name who sending this event

	// Keeping variable for making local events inside current process
	// Example: Docker Events, Balancer Events etc..
	LocalVar	interface{}			`json:"-" toml:"-" yaml:"-"`
}

var (
	events			=	make(map[string][]func(*Event))
	log_from_event	=	"Event handling and firing "
	NetworkEmitCB		func(e *Event, path *tree_graph.Path)tree_lib.TreeError
)

func TriggerFromData(data []byte) {
	var (
		e	=	new(Event)
		err 	tree_lib.TreeError
	)
	err.From = tree_lib.FROM_TRIGGER_FROM_DATA
	err.Err = ffjson.Unmarshal(data, e)
	if !err.IsNull() {
		tree_log.Error(log_from_event, err.Error())
		return
	}
	Trigger(e)
}

func TriggerWithData(name string, data []byte) {
	var (
		e	=	new(Event)
	)

	e.Name = name
	e.Data = data[:] // Sending by slice reference
	Trigger(e)
}

func Trigger(e *Event) {
	// If we don't have event with this name just returning
	// but if we have, then calling concurrent functions for handling event
	if funcs, ok :=events[e.Name]; ok {
		for _, f :=range funcs {
			go f(e)
		}
	}
}

// Set new event handler
func ON(name string, f func(*Event)) {
	events[name] = append(events[name], f)
}

// Delete event handler function from handlers list
func OFF(name string, f func(*Event)) {
	if funcs, ok :=events[name]; ok {
		for i, ff :=range funcs {
			if reflect.ValueOf(f).Pointer() == reflect.ValueOf(ff).Pointer() {
				// Deleting function by index
				events[name] = events[name][:i+copy(events[name][i:], events[name][i+1:])]
				break
			}
		}
	}
}

// Deleting full event with all handlers from list
func Delete(name string) {
	if _, ok :=events[name]; !ok {
		return
	}

	delete(events, name)
}

// Shortcut network event emitter callback
func Emit(ev *Event, p *tree_graph.Path) tree_lib.TreeError {
	return NetworkEmitCB(ev, p)
}