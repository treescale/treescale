package tree_net
import (
	"tree_graph"
	"tree_event"
	"tree_node/node_info"
	"net"
	"bytes"
	"tree_log"
	"github.com/pquerna/ffjson/ffjson"
	"encoding/binary"
	"fmt"
)

var (
	api_connections		=	make(map[string]*net.TCPConn)
)

const (
	// This is just a random string, using to notify Parent or Child Node that one of them going to close connection
	CLOSE_CONNECTION_MARK = "***###***"
)


func Start() {
	go ListenParent()
	go ChildListener(1000)
}

func handle_message(is_api, from_parent bool, msg []byte) (err error) {
	var (
		body_index	int
		path		tree_graph.Path
		node_names	[]string
	)
	body_index, path, err = tree_graph.PathFromMessage(msg)
	if err != nil {
		return
	}

	// If we got this data on node, then just firing event
	// don't meed to check is it right data received or not
	go tree_event.TriggerFromData(msg[body_index:])

	if is_api {
		// If message came from API then it need's to be handled only on this node
		//	then if there would be path to send , handler will send it from event callback
		return
	}

	if from_parent {
		node_names = path.ExtractNames(node_info.CurrentNodeInfo, node_info.ChildsNodeInfo)
	} else {
		node_names = path.ExtractNames(node_info.CurrentNodeInfo, node_info.ParentNodeInfo, node_info.ChildsNodeInfo...)
	}

	err = SendToNames(msg[body_index:], &path, node_names...)

	return
}

func SendToNames(data []byte, path *tree_graph.Path, names...string) (err error) {
	for _, n :=range names {
		var send_conn *net.TCPConn
		send_conn = nil

		if n_conn, ok := child_connections[n]; ok && n_conn != nil {
			send_conn = n_conn
		} else {
			if parent_name == n && parentConnection != nil {
				send_conn = parentConnection
			} else {
				if api_conn, ok := api_connections[n]; ok && api_conn != nil {
					send_conn = api_conn
				}
			}
		}

		if send_conn != nil {
			err = SendToConn(data, path, send_conn)
			if err != nil {
				tree_log.Error("Send to Node Names", err.Error())
			}
		}
	}
	return
}

func SendToConn(data []byte, path *tree_graph.Path, conn *net.TCPConn) (err error) {
	var (
		p_data	[]byte
		p_len	=	make([]byte, 4)
		msg_len	=	make([]byte, 4)
		buf		=	bytes.Buffer{}
	)
	p_data, err = ffjson.Marshal(path)
	binary.LittleEndian.PutUint32(p_len, uint32(len(p_data)))
	binary.LittleEndian.PutUint32(msg_len, uint32(len(p_data)) + uint32(len(data)))

	buf.Write(msg_len)
	buf.Write(p_len)
	buf.Write(p_data)
	buf.Write(data)

	_, err = conn.Write(buf.Bytes())
	return
}

func TcpConnect(ip string, port int) (conn *net.TCPConn, err error) {
	var (
		tcpAddr *net.TCPAddr
	)
	tcpAddr, err = net.ResolveTCPAddr("tcp", fmt.Sprintf("%s:%d", ip, port))
	if err != nil {
		return
	}

	conn, err = net.DialTCP("tcp", nil, tcpAddr)
	return
}