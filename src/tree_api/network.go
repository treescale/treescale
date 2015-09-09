package tree_api
import "net"

var (
	// Map for API connections with their names
	ApiConnections	= make(map[string]*net.TCPConn)
)

func HandleApiConnection(name string, conn *net.TCPConn) {
	// TODO: Trigger about new API connection



	// TODO: Trigger about new API connection close
}