package tree_net

import "net"

// This file contains functionality for handling child connections

var (
	child_connections	=	make(map[string]*net.TCPConn)
	log_from_parent		=	"Child connection handler"
)