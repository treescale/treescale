package tree_graph

import (
	"tree_db"
	tree_path "tree_graph/path"
	"tree_lib"
	"tree_node/node_info"
)
var (
	check =			make(map[string]bool)
	check_group =	make(map[string]bool)
	check_node =	make(map[string]bool)
	check_tag =		make(map[string]bool)
)

func GroupPath(from_node, group_name string) (map[string][]string, tree_lib.TreeError){
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
	for _, a := range nodes_in_group {
		check[a] = true
	}
	path, err = NodePath(from_node, group_name, true)
	if !err.IsNull() {
		return nil, err
	}
	return path, err
}

func NodePath(from_node, node_name string, isgroup bool) (map[string][]string, tree_lib.TreeError){
	var (
		node					string
		err						tree_lib.TreeError
		path =					make(map[string][]string)
		path1 					[]string
		relations = 			make(map[string][]string)
		nodes  					[]string
		from = 					make(map[string]string)
	)
	err.From = tree_lib.FROM_NODE_PATH
	nodes, err = tree_db.ListNodeNames()
	if !err.IsNull() {
		return nil, err
	}
	for _, a := range nodes {
		relations[a], err = tree_db.GetRelations(a)
		if !err.IsNull() {
			return nil, err
		}
	}

	from, node = bfs(from_node, node_name, relations, isgroup)

	for len(from) > 0 && node != from_node {
		path1 = append(path1, node)
		node = from[node]
	}
	path1 = append(path1, from_node)

	for i := len(path1)-1; i>0; i-- {
		path[path1[i]] = append(path[path1[i]], path1[i-1])
	}

	return path, err
}

func TagPath(from_node, tag_name string) (map[string][]string, tree_lib.TreeError){
	var (
		err						tree_lib.TreeError
		path =					make(map[string][]string)
		nodes_by_tagname 		[]string
		paths =					make(map[string]map[string][]string)
	)
	err.From = tree_lib.FROM_TAG_PATH
	nodes_by_tagname, err = tree_db.GetNodesByTagName(tag_name)
	if !err.IsNull() {
		return nil, err
	}
	for _, a := range nodes_by_tagname {
		paths[a], err = NodePath(from_node, a, false)
		if !err.IsNull() {
			return nil, err
		}
	}
	path = merge(paths, nil, nil)
	return path, err
}

func merge(nodes_path map[string]map[string][]string, groups_path map[string]map[string][]string, tags_path map[string]map[string][]string) (path map[string][]string){
	path = make(map[string][]string)
	for _, a := range nodes_path {
		for i, b := range a{
			for _, c := range b {
				path[i] = append(path[i], c)
			}
		}
	}
	if len(groups_path) > 0 {
		for _, a := range groups_path {
			for i, b := range a{
				for _, c := range b {
					path[i] = append(path[i], c)
				}
			}
		}
	}
	if len(tags_path) > 0 {
		for _, a := range tags_path {
			for i, b := range a{
				for _, c := range b {
					path[i] = append(path[i], c)
				}
			}
		}
	}
	return
}

func Check() (err tree_lib.TreeError) {
	var (
		nodes_info []node_info.NodeInfo
		node_names  []string
	)
	nodes_info, err = tree_db.ListNodeInfos()
	if !err.IsNull() {
		return err
	}
	node_names, err = tree_db.ListNodeNames()
	if !err.IsNull() {
		return err
	}
	for _, a := range node_names {
		check_node[a] = true
	}
	for _, a := range nodes_info {
		for _, b := range a.Groups {
			check_group[b] = true
		}
		for _, b := range a.Tags {
			check_tag[b] = true
		}
	}
	return err
}

func bfs(from_node, end string, nodes map[string][]string, isgroup bool) (map[string]string, string){
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
				if !isgroup {
					if n == end {
						return from, n
					}
				} else {
					if check[n]{
						return from, n
					}
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

func GetPath(from_node string, nodes []string, tags []string, groups []string) (*tree_path.Path, tree_lib.TreeError){
	var (
		err					tree_lib.TreeError
		path =				new(tree_path.Path)
		nodes_path =		make(map[string]map[string][]string)
		tags_path =			make(map[string]map[string][]string)
		groups_path = 		make(map[string]map[string][]string)
		final_path =		make(map[string][]string)
	)
	err.From = tree_lib.FROM_GET_PATH
	err = Check()
	if !err.IsNull(){
		return nil, err
	}
	for _, a := range nodes {
		if check_node[a] {
			nodes_path[a], err = NodePath(from_node, a, false)
			if !err.IsNull() {
				return nil, err
			}
		}
	}
	for _, a := range groups {
		if check_group[a] {
			groups_path[a], err = GroupPath(from_node, a)
			if !err.IsNull() {
				return nil, err
			}
			check = make(map[string]bool)
		}
	}
	for _, a := range tags {
		if check_tag[a] {
			tags_path[a], err = TagPath(from_node, a)
			if !err.IsNull() {
				return nil, err
			}
		}
	}
	final_path = merge(nodes_path, groups_path, tags_path)
	path.NodePaths = final_path
	path.Tags = tags
	path.Groups = groups
	return path, err
}