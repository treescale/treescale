package tree_db

import (
	"github.com/siddontang/ledisdb/ledis"
	"github.com/siddontang/ledisdb/config"
	"tree_log"
	"os"
	"tree_event"
)

var (
	DB_DIR		=	"/etc/treescale/db"
	tree_db			*ledis.Ledis
	db_conf		=	new(config.Config)
	log_from_db	=	"Tree Database"

	// DB identifiers created by init function, and keeping as a shortcuts
	DATABASES	=	make(map[int]*ledis.DB)
)

const (
	// Keeping different database lists
	DB_NODE			=	0
	DB_BALANCER		=	1
	DB_RANDOM		=	2  // This will hold random data with Key -> Value []byte
	DB_GROUP		=	3  // Database with group name keys and node list value (t1, t2, ...) strings.Join(node_list, ",")
	DB_TAG			=	4  // Database with tag name keys and node list value (t1, t2, ...) strings.Join(node_list, ",")
	DB_RELATIONS	=	5  // Database for storing node relations (parent or child connections) strings.Join(node_list, ",")
)

func init() {
	DATABASES[DB_NODE] = nil
	DATABASES[DB_BALANCER] = nil
	DATABASES[DB_RANDOM] = nil
	DATABASES[DB_TAG] = nil
	DATABASES[DB_GROUP] = nil
	DATABASES[DB_RELATIONS] = nil

	// Configuring Database
	db_conf.DataDir = DB_DIR
	db_conf.AccessLog = "" // don't need access log
	db_conf.Addr = "" // don't need server to run
	db_conf.SlaveOf = ""
	db_conf.Readonly = false

	// default databases number
	db_conf.Databases = len(DATABASES)
	db_conf.UseReplication = false
	db_conf.Snapshot.MaxNum = 1
	db_conf.DBName = "goleveldb"
	var err error
	tree_db, err = ledis.Open(db_conf)
	if err != nil {
		tree_log.Error(log_from_db, " unable to open database", err.Error())
		tree_db = nil
		os.Exit(1) // Without database we can't keep and share configurations, so program should be exited
	}

	// Setting databases
	for _, d :=range []int{DB_NODE, DB_BALANCER, DB_RANDOM, DB_GROUP, DB_TAG, DB_RELATIONS} {
		DATABASES[d], err = tree_db.Select(d)
		if err != nil {
			tree_log.Error(log_from_db, " unable to select 'node' database", err.Error())
			tree_db = nil
			os.Exit(1) // Without database we can't keep and share configurations, so program should be exited
		}
	}

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