package tree_db

import (
	"errors"
	"github.com/siddontang/ledisdb/ledis"
)

func CheckDB(db int) bool {
	if d, ok :=DATABASES[db]; ok && d != nil {
		return true
	}
	return false
}

func Set(db int, key, value []byte) (err error) {
	if !CheckDB(db) {
		err = errors.New("Node database is not selected, please select before making query")
		return
	}
	err = DATABASES[db].Set(key, value)
	return
}

func Get(db int, key []byte) (ret []byte, err error) {
	if !CheckDB(db) {
		err = errors.New("Node database is not selected, please select before making query")
		return
	}

	ret, err = DATABASES[db].Get(key)
	return
}


// Getting all keys as a callback, because there could be a lot of records and we don't want to fill RAM
// On every callback function you will receive up to 500 keys, if you will receive 0 keys then it is the last time
func AllKeys(db int, match string, cb func([][]byte)bool) (err error) {
	if !CheckDB(db) {
		err = errors.New("Node database is not selected, please select before making query")
		return
	}

	if cb == nil {
		err = errors.New("Set callback for getting keys on every iteration")
		return
	}

	var (
		cursor			[]byte
		cb_data			[][]byte
		first		=	true
	)
	cursor = nil

	for {
		if !first && len(cb_data) == 0 {
			break
		}
		cb_data, err = DATABASES[db].Scan(ledis.KV, cursor, 500, false, match)
		if err != nil {
			return
		}
		if !cb(cb_data) {
			break
		}
		if len(cb_data) == 0 {
			break
		}
		cursor = cb_data[len(cb_data) - 1]  // getting last key as a cursor for the next time
		first = false
	}

	return
}