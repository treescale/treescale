package tree_event

import (
	"tree_graph"
	"tree_log"
	"github.com/pquerna/ffjson/ffjson"
)


type Event struct {
	Name		string				`json:"name" toml:"name" yaml:"name"`
	Path		tree_graph.Path		`json:"path" toml:"path" yaml:"path"`
	Data		[]byte				`json:"data" toml:"data" yaml:"data"`
}

var (
	events			=	make(map[string][]func(*Event))
	log_from_event	=	"Event handling and firing "
)

func TriggerFromData(data []byte) {
	var (
		e	=	new(Event)
		err 	error
	)

	err = ffjson.Unmarshal(data, e)
	if err != nil {
		tree_log.Error(log_from_event, err.Error())
		return
	}
	Trigger(e)
}

func TriggerWithData(name string, data []byte, path *tree_graph.Path) {
	var (
		e	=	new(Event)
	)

	if path != nil {
		e.Path = (*path)
	}
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
			if f == ff {
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