package main

import (
	"tree_console"
	"tree_lib"
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