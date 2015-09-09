package tree_net

var (
	// This should be set before starting Networking
	// Because it will make some network handshake based on it and will make DB requests based on this name
	CurrentNodeName		string

)

const (
	// This is just a random string, using to notify Parent or Child Node that one of them going to close connection
	CLOSE_CONNECTION_MARK = "***###***"
)