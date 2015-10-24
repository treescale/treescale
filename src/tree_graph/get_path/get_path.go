package get_path


import(
	"math/big"
	"tree_lib"
	"tree_node/node_info"
	"tree_db"
	"fmt"
	"tree_graph"
)
var (
	check_group =		make(map[string]bool)
	check_node =		make(map[string]bool)
	check_tag =			make(map[string]bool)
	targets 			[]string
	nodes_info 			[]node_info.NodeInfo
	node_values =		make(map[string]int64)
)


func init() {
	// Initializing Path functions, because otherwise it is giving import cycle error
	tree_graph.CalcPath = CalculatePath
	tree_graph.GetPathValue = GetValue
	tree_graph.CalcApiPath = CalculateApiPath
}

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

func NodePath (from_node string, to_node string) (path *big.Int, err tree_lib.TreeError) {
	var (
		relations = 			make(map[string][]string)
		from = 					make(map[string]string)
		node					string
	)
	err.From = tree_lib.FROM_NODE_PATH
	path = big.NewInt(1)
	for _, a := range nodes_info {
		relations[a.Name], err = tree_db.GetRelations(a.Name)
		if !err.IsNull() {
			return
		}
	}
	value := big.Int{}
	from, node = bfs(from_node, to_node, relations)
	for len(from) > 0 && node != from_node {
		value.SetInt64(node_values[node])
		path.Mul(path, &value)
		node = from[node]
	}
	return
}

func GroupPath (from_node, group_name string) (map[string]*big.Int, tree_lib.TreeError){
	var (
		path = 			make(map[string]*big.Int)
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
			targets = append(targets, n)
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

func TagPath(from_node, tag_name string) (map[string]*big.Int, tree_lib.TreeError){
	var (
		err						tree_lib.TreeError
		path =					make(map[string]*big.Int)
		nodes_by_tagname 		[]string
	)
	err.From = tree_lib.FROM_TAG_PATH
	nodes_by_tagname, err = tree_db.GetNodesByTagName(tag_name)
	if !err.IsNull() {
		return nil, err
	}
	for _, n := range nodes_by_tagname {
		if check_node[n] {
			targets = append(targets, n)
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

func merge (path []map[string]*big.Int) (*big.Int) {
	var (
		value =	big.Int{}
	)
	final_path := big.NewInt(1)
	for _, a := range path {
		for _, p := range a {
			final_path = tree_lib.LCM(final_path, p)
		}
	}
	for _, n := range targets {
		value.SetInt64(node_values[n])
		final_path.Mul(final_path, &value)
	}
	return final_path
}

// Getting path value
// If path value is nil then just calculating it , otherwise just returning existing path
func GetValue(p *tree_graph.Path) (final_path *big.Int, err tree_lib.TreeError) {
	if p.Path == nil {
		final_path, err = p.CalculatePath()
	}
	final_path = p.Path
	return
}

func CalculatePath(p *tree_graph.Path) (final_path *big.Int, err tree_lib.TreeError) {
	var (
		node_path 		=		make(map[string]*big.Int)
		path					[]map[string]*big.Int
	)
	final_path = big.NewInt(1)
	err.From = tree_lib.FROM_GET_PATH
	nodes_info, err = tree_db.ListNodeInfos()
	if !err.IsNull() {
		return
	}

	Check()

	for _, n := range nodes_info {
		node_values[n.Name] = n.Value
	}
	if len(p.Nodes) > 0 && p.Nodes[0] == "*" {
		var val big.Int
		final_path = big.NewInt(1)
		for _, n := range nodes_info {
			val = big.Int{}
			val.SetInt64(n.Value)
			final_path.Mul(final_path, &val)
			final_path.Mul(final_path, &val)
		}
		p.Path = final_path
		return
	}

	for _, n := range p.Nodes {
		if check_node[n] {
			targets = append(targets, n)
			node_path[n], err = NodePath(p.From, n)
			if !err.IsNull() {
				return
			}
		}else {
			fmt.Println("there is no server with name ", n)
			fmt.Println("ignoring server ", n)
		}
	}
	path = append(path, node_path)
	for _, g := range p.Groups {
		if check_group[g] {
			node_path, err = GroupPath(p.From, g)
			if !err.IsNull() {
				return
			}
			path = append(path, node_path)
		} else {
			fmt.Println("there is no group with name ", g)
			fmt.Println("ignoring group ", g)
		}
	}
	for _, t := range p.Tags {
		if check_tag[t] {
			node_path, err = TagPath(p.From, t)
			if !err.IsNull() {
				return
			}
			path = append(path, node_path)
		}else {
			fmt.Println("there is no tag with name ", t)
			fmt.Println("ignoring tag ", t)
		}
	}
	final_path = merge(path)
	nodes_info = nil
	targets = []string{}
	p.Path = final_path

	//if path contains node, then final_path divides to value of node
	//if node is a target, then final path divides square of value of node
	return
}

func CalculateApiPath (p *tree_graph.Path, api *big.Int)(final_path *big.Int, err tree_lib.TreeError) {
	final_path, err = CalculatePath(p)
	if !err.IsNull() {
		return
	}
	final_path.Div(final_path, big.NewInt(node_values[p.Nodes[0]]))
	final_path.Mul(final_path, api)
	final_path.Mul(final_path, api)
	return
}