package tree_net
import "tree_graph"

var (
	// This should be set before starting Networking
	// Because it will make some network handshake based on it and will make DB requests based on this name
	CurrentNodeName		string

)

const (
	// This is just a random string, using to notify Parent or Child Node that one of them going to close connection
	CLOSE_CONNECTION_MARK = "***###***"
)

func handle_message(msg []byte) (err error) {
	var (
		body_index	int
		path		tree_graph.Path
	)
	body_index, path, err = tree_graph.PathFromMessage(msg)
	if err != nil {
		return
	}


	return
}