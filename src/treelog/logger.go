package treelog

import (
	"log"
)

var (
	LogFile = "/var/log/treescale/treescale.log"
)

func Error(from string, messages...string) {
	log.Println(from, ": ", messages)
}

func Info(from, message string) {
	log.Println(from, ": ", message)
}