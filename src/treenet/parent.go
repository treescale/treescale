package treenet
import (
	"net"
	"fmt"
	"treelog"
	"treenode/nodeinfo"
	"treelib"
	"strings"
	"treeapi"
)

// This file contains functionality for handling parent connections

var (
	parentConnection		*net.TCPConn
	listener				*net.TCPListener

	log_from			=	"Parent connection handler"
)

func listen_parent() (err error) {
	var (
		addr	*net.TCPAddr
		conn	*net.TCPConn
	)

	// If port is not set, setting it to default 8888
	if nodeinfo.CurrentNodeinfo.TreePort == 0 {
		nodeinfo.CurrentNodeinfo.TreePort = 8888
	}

	addr, err = net.ResolveTCPAddr("tcp", fmt.Sprintf("%s:%d", nodeinfo.CurrentNodeinfo.TreeIp, nodeinfo.CurrentNodeinfo.TreePort))
	if err != nil {
		treelog.Error(log_from, "Network Listen function", err.Error())
		return
	}

	listener, err = net.ListenTCP("tcp", addr)
	if err != nil {
		treelog.Error(log_from, "Network Listen function", err.Error())
		return
	}

	for {
		conn, err = listener.AcceptTCP()
		if err != nil {
			treelog.Error(log_from, err.Error())
			return
		}

		// Handle Parent connection
		go handle_api_or_parent_connection(conn)
	}
	return
}

func handle_api_or_parent_connection(conn *net.TCPConn) {
	defer conn.Close()  // Connection should be closed, after return this function
	var (
		err 		error
		msg_data	[]byte
		conn_name	string
	)

	// Making basic handshake to check the API validation
	// Connected Parent receiving name of the child(current node) and checking is it valid or not
	// if it is valid name then parent sending his name as an answer
	// otherwise it sending CLOSE_CONNECTION_MARK and closing connection

	_, err = treelib.SendMessage([]byte(nodeinfo.CurrentNodeinfo.Name), conn)
	if err != nil {
		treelog.Error(log_from, err.Error())
		return
	}

	msg_data, err = treelib.ReadMessage(conn)
	if err != nil {
		treelog.Error(log_from, err.Error())
		return
	}
	conn_name = string(msg_data)
	if conn_name == CLOSE_CONNECTION_MARK {
		treelog.Info(log_from, "Connection closed by parent node. Bad tree network handshake ! ", "Parent Addr: ", conn.RemoteAddr().String())
		return
	}

	if strings.Contains(conn_name, treeapi.API_NAME_PREFIX) {
		treeapi.HandleApiConnection(conn_name, conn)
		return
	}

	parentConnection = conn

	// TODO: Trigger about new parent connection with parent name

	// Listening parent events
	for {
		msg_data, err = treelib.ReadMessage(conn)
	}

	// TODO: Trigger about parent connection close
}