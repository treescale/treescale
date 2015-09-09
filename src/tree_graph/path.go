package tree_graph

import (
	"encoding/binary"
	"github.com/pquerna/ffjson/ffjson"
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