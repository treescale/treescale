package tree_db

import (
	"tree_log"
	"os"
	"tree_event"
	"tree_lib"
	"github.com/boltdb/bolt"
)

var (
	DB_DIR		=	tree_lib.GetEnv("TREE_DB_PATH", "/etc/treescale/tree.db")
	tree_db			*bolt.DB
	log_from_db	=	"Tree Database"

// Keeping different database lists
	DB_NODE			=	[]byte("node")
	DB_BALANCER		=	[]byte("balancer")
	DB_RANDOM		=	[]byte("random")  // This will hold random data with Key -> Value []byte
	DB_GROUP		=	[]byte("group")  // Database with group name keys and node list value (t1, t2, ...) strings.Join(node_list, ",")
	DB_TAG			=	[]byte("tag")  // Database with tag name keys and node list value (t1, t2, ...) strings.Join(node_list, ",")
	DB_RELATIONS	=	[]byte("relations")  // Database for storing node relations (parent or child connections) strings.Join(node_list, ",")
)


func init() {
	var err error
	tree_db, err = bolt.Open(DB_DIR, 0600, nil)
	if err != nil {
		tree_log.Error(log_from_db, " unable to open database", err.Error())
		tree_db = nil
		os.Exit(1) // Without database we can't keep and share configurations, so program should be exited
	}

	// creating Buckets in database
	tree_db.Update(func(tx *bolt.Tx) error{
		// Setting databases
		for _, d :=range [][]byte{DB_NODE, DB_BALANCER, DB_RANDOM, DB_GROUP, DB_TAG, DB_RELATIONS} {
			_, err := tx.CreateBucketIfNotExists(d)
			if err != nil {
				return err
			}
		}
		return nil
	})

	// Closing database before program will be exited
	// Just in case if program exiting force or we don't want to make dead lock
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

func LoadFromDumpPath(path string) (err error) {
	tree_lib.CopyFile(path, DB_DIR)
	return
}

func DumpDBPath(path string) (err error) {
	tree_lib.CopyFile(DB_DIR, path)
	return
}