package tree_net

import (
	"net"
	"tree_node/node_info"
	"tree_lib"
	"tree_log"
	"time"
	"tree_event"
)

// This file contains functionality for handling child connections

var (
	child_connections	=	make(map[string]*net.TCPConn)
	log_from_parent		=	"Child connection handler"
)

func ChildListener(interval time.Duration) {

	// TODO: this logic should be changed to evented, start reconnecting if child disconnected or there is connection error

	for {
		// If we have connected to childs that doesn't exists in current child list
		// then deleting them and closing connection
		for n, c :=range child_connections {
			if _, ok :=node_info.ChildsNodeInfo[n]; !ok {
				c.Close()
				delete(child_connections, n)
			}
		}

		for n, _ :=range node_info.ChildsNodeInfo {
			ChildConnect(n)
		}

		time.Sleep(time.Millisecond * interval)
	}
}

func ChildConnect(name string) (err error) {
	// If we don't have this name in childs info map then just returning
	if _, ok := node_info.ChildsNodeInfo[name]; !ok {
		return
	}

	// If we have already connected to this child then just returning
	if c, ok := child_connections[name]; ok && c != nil {
		return
	}

	var (
		conn			*net.TCPConn
		ch_name_data	[]byte
		msg				[]byte
	)

	conn, err = TcpConnect(node_info.ChildsNodeInfo[name].TreeIp, node_info.ChildsNodeInfo[name].TreePort)
	if err != nil {
		tree_log.Error(log_from_parent, " child_connect -> ", name, " ", err.Error())
		return
	}
	defer conn.Close()

	ch_name_data, err = tree_lib.ReadMessage(conn)
	if err != nil {
		tree_log.Error(log_from_parent, " child handshake -> ", name, " ", err.Error())
		return
	}

	// If name revieved from connection not same name as we have
	// Then we connected to wrong server, just returning
	// defer will close connection
	if string(ch_name_data) != name {
		tree_lib.SendMessage([]byte(CLOSE_CONNECTION_MARK), conn)
		return
	}

	_, err = tree_lib.SendMessage([]byte(node_info.CurrentNodeInfo.Name), conn)
	if err != nil {
		tree_log.Error(log_from_parent, " child handshake sending current name to -> ", name, " ", err.Error())
		return
	}

	child_connections[name] = conn

	tree_event.TriggerWithData(tree_event.ON_CHILD_CONNECTED, ch_name_data, nil)

	for {
		msg, err = tree_lib.ReadMessage(conn)
		if err != nil {
			tree_log.Error(log_from_parent, " reading data from child -> ", name, " ", err.Error())
			break
		}

		handle_message(false, false, msg)
	}

	child_connections[name] = nil
	delete(child_connections, name)

	tree_event.TriggerWithData(tree_event.ON_CHILD_DISCONNECTED, ch_name_data, nil)

	return
}