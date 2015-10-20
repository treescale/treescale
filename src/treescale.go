package main

import (
	"tree_console"
	"tree_lib"

	// Just to load path functions
	_ "tree_graph/get_path"
	_ "net/http/pprof"
)

const (
	DEFAULT_PROCESS_ID_FILE =	"/etc/treescale/pid"
)

var (
	PID_FILE	=	tree_lib.GetEnv("TREE_PID_FILE", DEFAULT_PROCESS_ID_FILE)
)

func main() {
	tree_console.HandleConsoleArgs()
}