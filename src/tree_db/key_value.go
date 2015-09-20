package tree_db

import (
	"github.com/boltdb/bolt"
)

func Set(db []byte, key, value []byte) (err error) {
	err = tree_db.Update(func(tx *bolt.Tx) error{
		b := tx.Bucket(db)
		return b.Put(key, value)
	})
	return
}

func Get(db []byte, key []byte) (ret []byte, err error) {
	err = tree_db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(db)
		ret = b.Get(key)
		return nil
	})
	return
}

func ForEach(db []byte, cb func([]byte, []byte)error) (err error) {
	err = tree_db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(db)
		b.ForEach(cb)
		return nil
	})
	return
}