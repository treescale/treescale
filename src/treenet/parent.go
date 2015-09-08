package treenet
import (
	"net"
	"fmt"
	"treelog"
	"treenode/nodeinfo"
)

// This file contains functionality for handling parent connections

var (
	parentConnection		*net.TCPConn
	listener				*net.TCPListener

	// Setting default listen IP and port
	listen_ip			=	""
	listen_port			=	8888

	log_from			=	"Parent connection handler"
)

func listen_parent() (err error) {
	var (
		addr	*net.TCPAddr
		conn	*net.TCPConn
	)

	addr, err = net.ResolveTCPAddr("tcp", fmt.Sprintf("%s:%d", listen_ip, listen_port))
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
	)

	// Making basic handshake to check the API validation
	// Connected Parent receiving name of the child(current node) and checking is it valid or not
	// if it is valid name then parent sending his name as an answer
	// otherwise it sending CLOSE_CONNECTION_MARK and closing connection

	_, err = SendMessage([]byte(nodeinfo.CurrentNodeinfo.Name), conn)
	if err != nil {
		treelog.Error(log_from, err.Error())
		return
	}

	msg_data, err = ReadMessage(conn)
	if err != nil {
		treelog.Error(log_from, err.Error())
		return
	}
	if string(msg_data) == CLOSE_CONNECTION_MARK {
		treelog.Info(log_from, "Connection closed by parent node. Bad tree network handshake ! ", "Parent Addr: ", conn.RemoteAddr().String())
		return
	}

	parentConnection = conn

	// TODO: Trigger about new parent connection with parent name

	// Listening parent events
	for {
		msg_data, err = ReadMessage(conn)
	}

	// TODO: Trigger about parent connection close
}