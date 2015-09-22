package tree_graph

import (
	"tree_db"
	tree_path "tree_graph/path"
)
var (
	check =		make(map[string]bool)
)
func GroupPath(from_node, group_name string) (map[string][]string, error){
	var (
		path = 			make(map[string][]string)
		err 			error
		nodes_in_group	[]string
	)
	nodes_in_group, err = tree_db.GetGroupNodes(group_name)
	if err != nil {
		return nil, err
	}
	for _, a := range nodes_in_group {
		check[a] = true
	}
	path, err = NodePath(from_node, group_name, true)
	if err != nil {
		return nil, err
	}
	return path, nil
}

func NodePath(from_node, node_name string, isgroup bool) (map[string][]string, error){
	var (
		node					string
		err						error
		path =					make(map[string][]string)
		path1 					[]string
		relations = 			make(map[string][]string)
		nodes  					[]string
		from = 					make(map[string]string)
	)

	nodes, err = tree_db.ListNodeNames()
	if err != nil {
		return nil, err
	}
	for _, a := range nodes {
		relations[a], err = tree_db.GetRelations(a)
		if err != nil {
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

	return path, nil
}

func TagPath(from_node, tag_name string) (map[string][]string, error){
	var (
		err						error
		path =					make(map[string][]string)
		nodes_by_tagname 		[]string
		paths =					make(map[string]map[string][]string)
	)
	nodes_by_tagname, err = tree_db.GetNodesByTagName(tag_name)
	if err != nil {
		return nil, err
	}
	for _, a := range nodes_by_tagname {
		paths[a], err = NodePath(from_node, a, false)
		if err != nil {
			return nil, err
		}
	}
	path = merge(paths, nil, nil)
	return path, nil
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

func GetPath(from_node string, nodes []string, tags []string, groups []string) (*tree_path.Path, error){
	var (
		err					error
		path =				new(tree_path.Path)
		nodes_path =		make(map[string]map[string][]string)
		tags_path =			make(map[string]map[string][]string)
		groups_path = 		make(map[string]map[string][]string)
		final_path =		make(map[string][]string)
	)
	for _, a := range nodes {
		nodes_path[a], err = NodePath(from_node, a, false)
		if err != nil {
			return nil, err
		}
	}
	for _, a := range groups {
		groups_path[a], err = GroupPath(from_node, a)
		if err != nil {
			return nil, err
		}
		check = make(map[string]bool)
	}
	for _, a := range tags {
		tags_path[a], err = TagPath(from_node, a)
		if err != nil {
			return nil, err
		}
	}
	final_path = merge(nodes_path, groups_path, tags_path)
	path.NodePaths = final_path
	path.Tags = tags
	path.Groups = groups
	return path, nil
}