package tree_db

import (
	"tree_log"
	"os"
	"tree_event"
	"tree_lib"
	"github.com/boltdb/bolt"
	"github.com/pquerna/ffjson/ffjson"
	"tree_node/node_info"
//	"tree_net
)

const (
	DEFAULT_DB_FILE	=	"/etc/treescale/tree.db"
)

var (
	DB_DIR		=	tree_lib.GetEnv("TREE_DB_PATH", DEFAULT_DB_FILE)
	tree_db			*bolt.DB
	log_from_db	=	"Tree Database"

// Keeping different database lists
	DB_NODE			=	[]byte("node")
	DB_BALANCER		=	[]byte("balancer")
	DB_REGISTRY		=	[]byte("registry")	// Containers registry
	DB_RANDOM		=	[]byte("random")  // This will hold random data with Key -> Value []byte
	DB_GROUP		=	[]byte("group")  // Database with group name keys and node list value (t1, t2, ...) strings.Join(node_list, ",")
	DB_TAG			=	[]byte("tag")  // Database with tag name keys and node list value (t1, t2, ...) strings.Join(node_list, ",")
	DB_RELATIONS	=	[]byte("relations")  // Database for storing node relations (parent or child connections) strings.Join(node_list, ",")
)


func init() {
	var err tree_lib.TreeError
	err.From = tree_lib.FROM_INIT
	tree_db, err.Err = bolt.Open(DB_DIR, 0600, nil)
	if !err.IsNull() {
		tree_log.Error(log_from_db, " unable to open database", err.Error())
		tree_db = nil
		os.Exit(1) // Without database we can't keep and share configurations, so program should be exited
	}

	// creating Buckets in database
	tree_db.Update(func(tx *bolt.Tx) (err error) {
		// Setting databases
		for _, d :=range [][]byte{DB_NODE, DB_BALANCER, DB_RANDOM, DB_GROUP, DB_TAG, DB_RELATIONS, DB_REGISTRY} {
			_, err = tx.CreateBucketIfNotExists(d)
			if err != nil {
				return err
			}
		}
		return nil
	})

	// Closing database before program will be exited
	// Just in case if program exiting force or we don't want to make dead lock
	tree_event.ON(tree_event.ON_UPDATE_NODE_INFO, func(e *tree_event.Event){
		var (
			info 				node_info.NodeInfo
			ev_info 			node_info.NodeInfo
			names 				[]string
			data 				[]byte

		)
		err.Err = ffjson.Unmarshal(e.Data, &ev_info)
		if !err.IsNull() {
			tree_log.Error(err.From,err.Error())
			return
		}
		info, err = GetNodeInfo(ev_info.Name)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return
		}
		if len(info.Name) > 0 {
			if len(ev_info.TreeIp) > 0 {
				info.TreeIp = ev_info.TreeIp
			}
			if ev_info.TreePort != -1 {
				info.TreePort = ev_info.TreePort
			}
			if len(ev_info.Childs[0]) > 0 {
				info.Childs = append(info.Childs, ev_info.Childs[0])
			}
			if len(ev_info.Childs[1]) > 0 {
				if g, ok := tree_lib.ArrayContains(info.Childs, ev_info.Childs[1]); ok {
					info.Childs = info.Childs[:g+copy(info.Childs[g:], info.Childs[g+1:])]
				}
			}
			if len(ev_info.Groups[0]) > 0 {
				info.Groups = append(info.Groups, ev_info.Groups[0])
			}
			if len(ev_info.Groups[1]) > 0 {
				if g, ok := tree_lib.ArrayContains(info.Groups, ev_info.Groups[1]); ok {
					info.Groups = info.Groups[:g+copy(info.Groups[g:], info.Groups[g+1:])]
				}
			}
			if len(ev_info.Tags[0]) > 0 {
				info.Tags = append(info.Tags, ev_info.Tags[0])
			}
			if len(ev_info.Tags[1]) > 0 {
				if g, ok := tree_lib.ArrayContains(info.Tags, ev_info.Tags[1]); ok {
					info.Tags = info.Tags[:g+copy(info.Tags[g:], info.Tags[g+1:])]
				}
			}
			data, err.Err = ffjson.Marshal(info)
			if !err.IsNull() {
				tree_log.Error(err.From, err.Error())
				return
			}
			err = Set(DB_NODE,[]byte(ev_info.Name),data)
			if !err.IsNull() {
				tree_log.Error(err.From, err.Error())
				return
			}
			names, err = ListNodeNames()
			if !err.IsNull(){
				tree_log.Error(err.From, err.Error())
				return
			}
			err = AddNodeToHisGroups(ev_info.Name)
			if !err.IsNull() {
				tree_log.Error(err.From, err.Error())
				return
			}
			err = AddNodeToHisTags(ev_info.Name)
			if !err.IsNull() {
				tree_log.Error(err.From, err.Error())
				return
			}
			for _, n := range names {
				err = SetRelations(n)
				if !err.IsNull() {
					tree_log.Error(err.From, err.Error())
					return
				}
			}
			if node_info.CurrentNodeInfo.Name == ev_info.Name {
				//tree_net.Restart()
			}
		} else {
			err = Set(DB_NODE,[]byte(ev_info.Name),e.Data)
			if !err.IsNull() {
				tree_log.Error(err.From, err.Error())
				return
			}
			names, err = ListNodeNames()
			if !err.IsNull(){
				tree_log.Error(err.From, err.Error())
				return
			}
			err = AddNodeToHisGroups(ev_info.Name)
			if !err.IsNull() {
				tree_log.Error(err.From, err.Error())
				return
			}
			err = AddNodeToHisTags(ev_info.Name)
			if !err.IsNull() {
				tree_log.Error(err.From, err.Error())
				return
			}
			for _, n := range names {
				err = SetRelations(n)
				if !err.IsNull() {
					tree_log.Error(err.From, err.Error())
					return
				}
			}
			if node_info.CurrentNodeInfo.Name == ev_info.Name {
				var e		*tree_event.Event
				e.Name = tree_event.ON_RESTART_NODE
				tree_event.Trigger(e)
			}
		}
	})

	tree_event.ON(tree_event.ON_PROGRAM_EXIT, func(e *tree_event.Event){
		CloseDB()
	})
}

func CloseDB() {
	if tree_db == nil {
		return
	}
	tree_db.Close()
}

func LoadFromDumpPath(path string) (err tree_lib.TreeError) {
	tree_lib.CopyFile(path, DB_DIR)
	return
}

func DumpDBPath(path string) (err tree_lib.TreeError) {
	tree_lib.CopyFile(DB_DIR, path)
	return
}