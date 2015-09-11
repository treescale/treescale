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
	node_db, balancer_db, random_db 	*ledis.DB
)

const (
	// Keeping different database lists
	DB_COUNT		=	3	// NEED TO BE CHANGED IF THERE IS NEW DATABASE
	DB_NODE			=	0
	DB_BALANCER		=	1
	DB_RANDOM		=	2  // This will hold random data with Key -> Value []byte
)

func init() {
	// Configuring Database
	db_conf.DataDir = DB_DIR
	db_conf.AccessLog = "" // don't need access log
	db_conf.Addr = "" // don't need server to run
	db_conf.SlaveOf = ""
	db_conf.Readonly = false

	// default databases number
	db_conf.Databases = DB_COUNT
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

	node_db, err = tree_db.Select(DB_NODE)
	if err != nil {
		tree_log.Error(log_from_db, " unable to select 'node' database", err.Error())
		tree_db = nil
		os.Exit(1) // Without database we can't keep and share configurations, so program should be exited
	}
	balancer_db, err = tree_db.Select(DB_BALANCER)
	if err != nil {
		tree_log.Error(log_from_db, " unable to select 'balancer' database", err.Error())
		tree_db = nil
		os.Exit(1) // Without database we can't keep and share configurations, so program should be exited
	}
	random_db, err = tree_db.Select(DB_RANDOM)
	if err != nil {
		tree_log.Error(log_from_db, " unable to select 'random' database", err.Error())
		tree_db = nil
		os.Exit(1) // Without database we can't keep and share configurations, so program should be exited
	}

	tree_event.ON(tree_event.ON_PROGRAM_EXIT, func(e *tree_event.Event){
		CloseDB()
	})
}

func CloseDB() (err error) {
	if tree_db == nil {
		return
	}
	err = tree_db.Close()
	return
}