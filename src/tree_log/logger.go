package tree_log

import (
	"log"
)

var (
	LogFile = "/var/log/treescale/treescale.log"
)

func Error(from string, messages...string) {
	log.Println(from, ": ", messages)
}

func Info(from string, messages...string) {
	log.Println(from, ": ", messages)
}