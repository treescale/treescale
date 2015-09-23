package main

import (
	"tree_console"
	_ "net/http/pprof"
	"log"
	"net/http"
)

func main() {
	go func() {
		log.Println(http.ListenAndServe("localhost:6060", nil))
	}()
	/*err := pprof.StartCPUProfile(os.Stdin)
	if err != nil {
		fmt.Println(err.Error())
	}*/
	tree_console.HandleConsoleArgs()
}