package tree_lib

import (
	"reflect"
)


func ArrayContains(array []interface{}, v interface{}) (int, bool) {
	if len(array) == 0 {
		return -1, false
	}

	if reflect.TypeOf(array[0]) != reflect.TypeOf(v) {
		return -1, false
	}

	for i, vc :=range array {
		if vc == v {
			return i, true
		}
	}

	return -1, false
}


// Function checks is 2 Arrays contains same element or not
// And returning indexes for both, with bool containing or not
func ArrayMatchElement(array1 []interface{}, array2 []interface{}) (int, int, bool) {
	if len(array1) == 0 || len(array2) == 0 {
		return -1, -1, false
	}

	if reflect.TypeOf(array1) != reflect.TypeOf(array2) {
		return -1, -1, false
	}

	for i, v :=range array1 {
		for j, h :=range array2 {
			if v == h {
				return i, j, true
			}
		}
	}

	return -1, -1, false
}