package tree_db

import (
	"github.com/boltdb/bolt"
	"tree_lib"
)

func Set(db []byte, key, value []byte) (err tree_lib.TreeError) {
	err.From = tree_lib.FROM_SET
	err.Err = tree_db.Update(func(tx *bolt.Tx) error{
		b := tx.Bucket(db)
		return b.Put(key, value)
	})
	return
}

func Get(db []byte, key []byte) (ret []byte, err tree_lib.TreeError) {
	err.From = tree_lib.FROM_GET
	err.Err = tree_db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(db)
		ret = b.Get(key)
		return nil
	})
	return
}

func ForEach(db []byte, cb func([]byte, []byte)error) (err tree_lib.TreeError) {
	err.From = tree_lib.FROM_FOREACH
	err.Err = tree_db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(db)
		b.ForEach(cb)
		return nil
	})
	return
}