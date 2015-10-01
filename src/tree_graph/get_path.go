package tree_graph


import(
	"math/big"
	"tree_lib"
	"tree_node/node_info"
	"tree_db"
	"fmt"
)
var (
	check_group =		make(map[string]bool)
	check_node =		make(map[string]bool)
	check_tag =			make(map[string]bool)
	targets =			make(map[string]bool)
	mark = 				make(map[string]bool)
	nodes_info 			[]node_info.NodeInfo
	node_values =		make(map[string]int64)
)
func Check() {
	for _, a := range nodes_info {
		check_node[a.Name] = true
	}
	for _, a := range nodes_info {
		for _, b := range a.Groups {
			check_group[b] = true
		}
		for _, b := range a.Tags {
			check_tag[b] = true
		}
	}
}

func NodePath (from_node string, to_node string) (path []string, err tree_lib.TreeError) {
	var (
		relations = 			make(map[string][]string)
		from = 					make(map[string]string)
		node					string
	)
	err.From = tree_lib.FROM_NODE_PATH

	for _, a := range nodes_info {
		relations[a.Name], err = tree_db.GetRelations(a.Name)
		if !err.IsNull() {
			return
		}
	}
	from, node = bfs(from_node, to_node, relations)
	for len(from) > 0 && node != from_node {
		path = append(path, node)
		node = from[node]
	}
	return
}

func GroupPath (from_node, group_name string) (map[string][]string, tree_lib.TreeError){
	var (
		path = 			make(map[string][]string)
		err 			tree_lib.TreeError
		nodes_in_group	[]string
	)
	err.From = tree_lib.FROM_GROUP_PATH
	nodes_in_group, err = tree_db.GetGroupNodes(group_name)
	if !err.IsNull() {
		return nil, err
	}
	for _, n := range nodes_in_group {
		if check_node[n] {
			targets[n] = true
			path[n], err = NodePath(from_node, n)
			if !err.IsNull() {
				return nil, err
			}
		}else {
			fmt.Println("there is no server with name ", n)
			fmt.Println("ignoring server ", n)
		}
	}
	return path, err
}

func TagPath(from_node, tag_name string) (map[string][]string, tree_lib.TreeError){
	var (
		err						tree_lib.TreeError
		path =					make(map[string][]string)
		nodes_by_tagname 		[]string
	)
	err.From = tree_lib.FROM_TAG_PATH
	nodes_by_tagname, err = tree_db.GetNodesByTagName(tag_name)
	if !err.IsNull() {
		return nil, err
	}
	for _, n := range nodes_by_tagname {
		if check_node[n] {
			targets[n] = true
			path[n], err = NodePath(from_node, n)
			if !err.IsNull() {
				return nil, err
			}
		}else {
			fmt.Println("there is no server with name ", n)
			fmt.Println("ignoring server ", n)
		}
	}
	return path, err
}
func bfs(from_node, end string, nodes map[string][]string) (map[string]string, string){
	frontier := []string{from_node}
	visited := map[string]bool{}
	next := []string{}
	from := map[string]string{}

	for 0 < len(frontier) {
		next = []string{}
		for _, node := range frontier {
			visited[node] = true
			for _, n := range bfs_frontier(node, nodes, visited) {
				next = append(next, n)
				from[n] = node
				if n == end {
					return from, n
				}
			}
		}
		frontier = next
	}
	return nil, ""
}

func bfs_frontier(node string, nodes map[string][]string, visited map[string]bool) []string {
	next := []string{}
	iter := func (n string) bool { _, ok := visited[n]; return !ok }
	for _, n := range nodes[node] {
		if iter(n) {
			next = append(next, n)
		}
	}
	return next
}

func merge (path []map[string][]string) (big.Int, tree_lib.TreeError) {
	var (
		err 					tree_lib.TreeError
	)
	err.From = tree_lib.FROM_MERGE
	final_path := *big.NewInt(1)
	for _, a := range path {
		for _, p := range a {
			for _, n := range p {
				if !mark[n] {
					value := big.NewInt(node_values[n])
					final_path.Mul(&final_path, value)
					mark[n] = true
					if targets[n] {
						final_path.Mul(&final_path, value)
					}
				}
			}
		}
	}
	return final_path, err
}
func GetPath(from_node string, nodes []string, tags []string, groups []string) (final_path big.Int, err tree_lib.TreeError) {
	var (
		node_path 		=		make(map[string][]string)
		path					[]map[string][]string
	)
	err.From = tree_lib.FROM_GET_PATH

	Check()

	nodes_info, err = tree_db.ListNodeInfos()
	if !err.IsNull() {
		return
	}
	for _, n := range nodes_info {
		node_values[n.Name] = n.Value
	}
	for _, n := range nodes {
		if check_node[n] {
			targets[n] = true
			node_path[n], err = NodePath(from_node, n)
			if !err.IsNull() {
				return
			}
		}else {
			fmt.Println("there is no server with name ", n)
			fmt.Println("ignoring server ", n)
		}
	}
	path = append(path, node_path)
	for _, g := range groups {
		if check_group[g] {
			node_path, err = GroupPath(from_node, g)
			if !err.IsNull() {
				return
			}
			path = append(path, node_path)
		} else {
			fmt.Println("there is no group with name ", g)
			fmt.Println("ignoring group ", g)
		}
	}
	for _, t := range nodes {
		if check_tag[t] {
			node_path, err = TagPath(from_node, t)
			if !err.IsNull() {
				return
			}
			path = append(path, node_path)
		}else {
			fmt.Println("there is no tag with name ", t)
			fmt.Println("ignoring tag ", t)
		}
	}
	final_path, err  = merge(path)
	nodes_info = nil
	return
}