package treenet
import (
	"net"
	"fmt"
	"treelog"
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

}