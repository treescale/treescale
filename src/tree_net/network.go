package tree_net
import (
	"tree_event"
	"tree_node/node_info"
	"net"
	"bytes"
	"tree_log"
	"github.com/pquerna/ffjson/ffjson"
	"encoding/binary"
	"fmt"
	"tree_lib"
	"tree_graph"
	"math/big"
	"tree_api"
"strings"
)

var (
	api_connections		=	make(map[*big.Int]*net.TCPConn)
)

const (
	// This is just a random string, using to notify Parent or Child Node that one of them going to close connection
	CLOSE_CONNECTION_MARK = "***###***"
)


func init() {
	// Adding event emmit callback
	tree_event.NetworkEmitCB = NetworkEmmit
	// Child listener should be running without any condition
	go ChildListener(1000)
}

func Start() {
	ListenParent()
}

func Stop() {
	if parentConnection != nil {
		parentConnection.Close()
		parentConnection = nil
	}

	if listener != nil {
		listener.Close()
		listener = nil
	}

	for n, c :=range child_connections {
		if c != nil {
			c.Close()
		}
		delete(child_connections, n)
	}
}

func Restart() {
	Stop()
	Start()
}

func handle_message(is_api, from_parent bool, msg []byte) (err tree_lib.TreeError) {
	var (
		body_index		int
		path			*big.Int
		msg_data	=	msg[body_index:]
	)
	err.From = tree_lib.FROM_HANDLE_MESSAGE
	body_index, path = tree_graph.PathValueFromMessage(msg)

	fmt.Println(path.String(), node_info.CurrentNodeValue.String())

	// If current node dividable to path, then it should execute this event
	if ok, _ := tree_lib.IsBigDividable(path, node_info.CurrentNodeValue); ok {
		go tree_event.TriggerFromData(msg[body_index:])
	}

	fmt.Println(string(msg_data))

	SendToPath(msg_data, path)

	return
}

func SendToPath(data []byte, path *big.Int) {
	// If path is negative, then this path is from API, so it contains API number also
	// Every API client have is own simple number but with negative value for splitting path
	if path.Sign() < 0 {
		for n, c :=range api_connections {
			if ok, _ := tree_lib.IsBigDividable(path, n); ok {
				SendToConn(data, c, path)
			}
		}
	}

	// First of all trying to send to parent
	if node_info.ParentNodeValue != nil && node_info.ParentNodeValue.Int64() > 0 {
		if ok, _ := tree_lib.IsBigDividable(path, node_info.ParentNodeValue); ok {
			SendToParent(data, path)
		}
	}

	for n, v :=range node_info.ChildsNodeValue {
		if v != nil && v.Int64() > 0 {
			if ok, _ := tree_lib.IsBigDividable(path, v); ok {
				SendToChild(data, n, path)
			}
		}
	}
}

// Parent connection should be only one, that's why we don't need to specify name
func SendToParent(data []byte, path *big.Int) {
	SendToConn(data, parentConnection, path)
}

func SendToChild(data []byte, name string, path *big.Int) {
	var (
		conn	*net.TCPConn
		ok		bool
	)

	// If we don't have node with this name connected
	// Then just returning from this function
	if conn, ok = child_connections[name]; !ok {
		return
	}

	SendToConn(data, conn, path)
}

func SendToConn(data []byte, conn *net.TCPConn, path *big.Int) {
	// making variable for combining send data
	var (
		err 				tree_lib.TreeError
		path_len_data	=	make([]byte, 4)
		msg_len_data	=	make([]byte, 4)
		path_data		=	path.Bytes()
		path_len		=	uint32(len(path_data))
		buf				=	bytes.Buffer{}
	)

	err.From = tree_lib.FROM_SEND_TO_CONN

	binary.LittleEndian.PutUint32(path_len_data, path_len)
	binary.LittleEndian.PutUint32(msg_len_data, path_len + uint32(len(data)) + uint32(4))

	buf.Write(msg_len_data)
	buf.Write(path_len_data)
	buf.Write(path_data)
	buf.Write(data)

	if conn != nil {
		_, err.Err = conn.Write(buf.Bytes())
		if !err.IsNull() {
			tree_log.Error(err.From, fmt.Sprintf("Error sending data to path [%s]", path.String()), err.Error())
		}
	}

	buf.Reset()
}

func TcpConnect(ip string, port int) (conn *net.TCPConn, err tree_lib.TreeError) {
	var (
		tcpAddr *net.TCPAddr
	)
	err.From = tree_lib.FROM_TCP_CONNECT
	tcpAddr, err.Err = net.ResolveTCPAddr("tcp", fmt.Sprintf("%s:%d", ip, port))
	if !err.IsNull() {
		return
	}

	conn, err.Err = net.DialTCP("tcp", nil, tcpAddr)
	return
}

func NetworkEmmit(e *tree_event.Event, path *tree_graph.Path) (err tree_lib.TreeError) {
	var (
		sdata		[]byte
		p			*big.Int
	)
	err.From = tree_lib.FROM_NETWORK_EMIT

	// Calling get value, because maybe some one will calculate this path before calling this functions
	// If path is not calculated yet, it will be automatically calculated in GetValue function
	p, err = path.GetValue()
	if !err.IsNull() {
		return
	}

	// If we emitting from API then we need to multiply path with connected node
	// For sending data through him
	if strings.Contains(node_info.CurrentNodeInfo.Name, tree_api.API_NAME_PREFIX) {
		p.Mul(p, node_info.ChildsNodeValue[path.From])
	}

	// If from not set, setting it before network sending
	if len(e.From) == 0 {
		e.From = node_info.CurrentNodeInfo.Name
	}

	sdata, err.Err = ffjson.Marshal(e)
	if !err.IsNull() {
		return
	}

	SendToPath(sdata, p)
	return
}