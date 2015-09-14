package path

import (
	"encoding/binary"
	"github.com/pquerna/ffjson/ffjson"
	"tree_node/node_info"
)

type Path struct {
	NodePaths		map[string][]string		`json:"node_paths" toml:"node_paths" yaml:"node_paths"`
	Groups			[]string				`json:"groups" toml:"groups" yaml:"groups"`
	Tags			[]string				`json:"tags" toml:"tags" yaml:"tags"`
}

func PathFromMessage(msg []byte) (body_index int, p Path, err error) {
	// First 4 bytes in message is a length of json encoded Path
	path_len := int(binary.LittleEndian.Uint32(msg[:4]))
	err = ffjson.Unmarshal(msg[4:path_len], &p)
	body_index = 4 + path_len
	return
}

func (path *Path) ExtractNames(current_node node_info.NodeInfo, nodes_info map[string]node_info.NodeInfo) (p_nodes []string) {
	var (
		ok			bool
	)

	if p_nodes, ok = path.NodePaths[current_node.Name]; ok {
		// deleting this path after getting it
		delete(path.NodePaths, current_node.Name)
	}

	for name, info :=range nodes_info {
		contains := false
		for   _, pt :=range path.Tags {
			for _, nt :=range info.Tags {
				if nt == pt {
					contains = true
					break
				}
			}

			if contains {
				break
			}
		}

		if !contains {
			for _, pg :=range path.Tags {
				for _, ng :=range info.Groups {
					if ng == pg {
						contains = true
						break
					}
				}

				if contains {
					break
				}
			}
		}

		if contains {
			duplicate := false
			for _, pn :=range p_nodes {
				if pn == name {
					duplicate = true
					break
				}
			}
			if !duplicate {
				p_nodes = append(p_nodes, name)
			}
		}
	}

	return
}