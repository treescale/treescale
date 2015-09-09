package tree_db

import (
	"github.com/HouzuoGuo/tiedot/db"
	"github.com/pquerna/ffjson/ffjson"
)

var (
	DB_DIR		=	"/etc/treescale/db"
	tree_db			*db.DB
)

func InitDB() (err error) {
	tree_db, err = db.OpenDB(DB_DIR)
	return
}


func CloseDB() (err error) {
	if tree_db == nil {
		return
	}
	err = tree_db.Close()
	return
}

func HaveCollection(name string) (have bool, err error) {
	if tree_db == nil {
		err = InitDB()
		if err != nil {
			return
		}
	}

	have = false

	for _, c :=range tree_db.AllCols() {
		if c == name {
			have = true
			return
		}
	}

	return
}

func GetCollection(name string) (col *db.Col, err error) {
	have := false
	err, have = HaveCollection(name);
	if err != nil {
		return
	}

	if !have {
		err = tree_db.Create(name)
		if err != nil {
			return
		}
	}

	col, err = tree_db.Use(name)
	return
}

func DropCollection(name string) (err error) {
	have := false
	err, have = HaveCollection(name);
	if err != nil {
		return
	}

	if !have {
		return
	}

	tree_db.Drop(name)
	return
}

func QueryCol(col *db.Col, query_str string) (query_result map[int]struct {}, err error) {
	var (
		query interface{}
	)

	err = ffjson.Unmarshal([]byte(query_str), &query)
	if err != nil {
		return
	}

	err = db.EvalQuery(query, col, &query_result)
	return
}

func Query(col_name, query string) (query_result map[int]struct {}, err error) {
	var (
		col	*db.Col
	)
	col, err = GetCollection(col_name)
	if err != nil {
		return
	}

	query_result, err = QueryCol(col, query)
	return
}