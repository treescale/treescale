package tree_event
import (
	"os"
	"os/signal"
	"syscall"
)


func init() {
	// firing event before program will exit
	c := make(chan os.Signal, 1)
	signal.Notify(c,
		syscall.SIGINT,
		syscall.SIGHUP,
		syscall.SIGTERM,
		syscall.SIGKILL,
		syscall.SIGQUIT,
		syscall.SIGTERM)
	go func(){
		<-c
		if funcs, ok :=events[ON_PROGRAM_EXIT]; ok {
			for _, f :=range funcs {
				f(nil)
			}
		}
		os.Exit(1)
	}()
}