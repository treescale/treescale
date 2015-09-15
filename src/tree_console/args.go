package tree_console
import (
	"os"
)

func HandleConsoleArgs() {
	TreeScaleCMD.SetArgs(os.Args[1:])
	TreeScaleCMD.Execute()
}