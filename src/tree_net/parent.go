package tree_net

import (
	"net"
	"tree_node/node_info"
	"tree_lib"
	"tree_log"
	"tree_event"
	"github.com/pquerna/ffjson/ffjson"
)

// This file contains functionality for handling child connections

var (
	child_connections	=	make(map[string]*net.TCPConn)
	log_from_parent		=	"Child connection handler"
)

func ChildsConnector() {
	// If we have connected to childs that doesn't exists in current child list
	// then deleting them and closing connection
	for n, c :=range child_connections {
		if _, ok :=node_info.ChildsNodeInfo[n]; !ok {
			c.Close()
			delete(child_connections, n)
		}
	}

	for n, _ :=range node_info.ChildsNodeInfo {
		ChildConnectCheck(n)
	}
}

func ChildConnectCheck(name string) {
	// If we don't have this name in childs info map then just returning
	if _, ok := node_info.ChildsNodeInfo[name]; !ok {
		return
	}

	// If we have already connected to this child then just returning
	if c, ok := child_connections[name]; ok && c != nil {
		return
	}

	go ChildConnect(name)
}

func ChildConnect(name string) (err tree_lib.TreeError) {
	var (
		conn			*net.TCPConn
		curr_data		[]byte
		ch_info_data	[]byte
		conn_node_info	node_info.NodeInfo
		msg				[]byte
	)
	err.From = tree_lib.FROM_CHILD_CONNECT
	conn, err = TcpConnect(node_info.ChildsNodeInfo[name].TreeIp, node_info.ChildsNodeInfo[name].TreePort)
	if !err.IsNull() {
		tree_log.Error(err.From, " child_connect -> ", name, " ", err.Error())
		return
	}
	defer conn.Close()

	ch_info_data, err = tree_lib.ReadMessage(conn)
	if !err.IsNull() {
		tree_log.Error(err.From, " child handshake -> ", name, " ", err.Error())
		return
	}

	err.Err = ffjson.Unmarshal(ch_info_data, &conn_node_info)
	if !err.IsNull() {
		tree_log.Error(err.From, " child handshake -> ", name, " ", err.Error())
		return
	}

	// If name revieved from connection not same name as we have
	// Then we connected to wrong server, just returning
	// defer will close connection
	if conn_node_info.Name != name {
		tree_lib.SendMessage([]byte(CLOSE_CONNECTION_MARK), conn)
		return
	}

	curr_data, err.Err = ffjson.Marshal(node_info.CurrentNodeInfo)
	if !err.IsNull() {
		tree_log.Error(err.From, " child handshake -> ", name, " ", err.Error())
		return
	}

	_, err.Err = tree_lib.SendMessage(curr_data, conn)
	if !err.IsNull() {
		tree_log.Error(err.From, " child handshake sending current info to -> ", name, " ", err.Error())
		return
	}

	child_connections[name] = conn

	tree_event.TriggerWithData(tree_event.ON_CHILD_CONNECTED, ch_info_data)

	for {
		msg, err = tree_lib.ReadMessage(conn)
		if !err.IsNull() {
			tree_log.Error(err.From, " reading data from child -> ", name, " ", err.Error())
			break
		}

		handle_message(false, false, msg)
	}

	child_connections[name] = nil
	delete(child_connections, name)

	tree_event.TriggerWithData(tree_event.ON_CHILD_DISCONNECTED, ch_info_data)

	return
}