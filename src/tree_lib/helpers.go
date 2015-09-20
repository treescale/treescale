package tree_lib

import (
	"reflect"
	"os"
	"time"
"math/rand"
	"io"
)


func GetEnv(name, def_value string) string {
	val := os.Getenv(name)
	if len(val) > 0 {
		return val
	}
	return def_value
}

func ArrayContains(array interface{}, v interface{}) (int, bool) {
	if reflect.TypeOf(array).Kind() != reflect.Slice && reflect.TypeOf(array).Kind() != reflect.Array {
		return -1, false
	}
	arr_val := reflect.ValueOf(array)

	if arr_val.Len() == 0 {
		return -1, false
	}

	// Testing element types
	if reflect.TypeOf(arr_val.Index(0).Interface()).Kind() != reflect.TypeOf(v).Kind() {
		return -1, false
	}

	// Trying to find value
	for i:=0; i< arr_val.Len(); i++ {
		if arr_val.Index(i).Interface() == v {
			return i, true
		}
	}

	return -1, false
}


// Function checks is 2 Arrays contains same element or not
// And returning indexes for both, with bool containing or not
func ArrayMatchElement(array1 interface{}, array2 interface{}) (int, int, bool) {
	if	(reflect.TypeOf(array1).Kind() != reflect.Slice && reflect.TypeOf(array1).Kind() != reflect.Array) ||
		(reflect.TypeOf(array2).Kind() != reflect.Slice && reflect.TypeOf(array2).Kind() != reflect.Array) {
		return -1, -1, false
	}

	v1 := reflect.ValueOf(array1)
	v2 := reflect.ValueOf(array2)

	if v1.Len() == 0 || v2.Len() == 0 {
		return -1, -1, false
	}

	if reflect.TypeOf(v1.Index(0).Interface()).Kind() != reflect.TypeOf(v2.Index(0).Interface()).Kind() {
		return -1, -1, false
	}

	for i:=0; i< v1.Len(); i++ {
		for j:=0; j< v1.Len(); j++ {
			if v1.Index(i).Interface() == v2.Index(j).Interface() {
				return i, j, true
			}
		}
	}

	return -1, -1, false
}


const letterBytes = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789!&$#*-_~()}{][|"

func RandomString(n int) string {
	rand.Seed(time.Now().UnixNano())
	b := make([]byte, n)
	for i := range b {
		b[i] = letterBytes[rand.Intn(len(letterBytes))]
	}
	return string(b)
}

func CopyFile(src, dst string) (err error) {
	var (
		db_f, new_db_f	*os.File
	)
	db_f, err = os.Open(src)
	if err != nil {
		return
	}
	defer db_f.Close()

	new_db_f, err = os.Create(dst)
	if err != nil {
		return
	}
	defer new_db_f.Close()

	_, err = io.Copy(new_db_f, db_f)
	return
}