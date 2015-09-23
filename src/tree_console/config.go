package tree_console
import (
	"tree_node/node_info"
	"github.com/spf13/cobra"
	"path/filepath"
	"strings"
	"github.com/BurntSushi/toml"
	"io/ioutil"
	"gopkg.in/yaml.v2"
	"github.com/pquerna/ffjson/ffjson"
	"tree_log"
	"tree_db"
	"os"
	"fmt"
	"tree_balancer"
	"tree_container/tree_docker"
	"tree_lib"
	"path"
)

const (
	log_from_config	=	"From config functionality"
)

type TreeScaleConf struct {
	SSH				map[string]SSHConfig						`toml:"ssh" json:"ssh" yaml:"ssh"`
	TreeNode		map[string]node_info.NodeInfo				`toml:"tree_node" json:"tree_node" yaml:"tree_node"`
	Balancer		map[string]tree_balancer.BalancerConfig		`toml:"balancer" json:"balancer" yaml:"balancer"`
	Registry		map[string]tree_docker.DockerRegistry		`toml:"registry" json:"registry" yaml:"registry"`
}

var (
	GLOBAL_CONFIG	TreeScaleConf
)

func ParseConfigFile(file string) (conf TreeScaleConf, err tree_lib.TreeError) {
	err.From = tree_lib.FROM_PARSE_CONFIG_FILE
	switch strings.Replace(filepath.Ext(file), ".", "", 1) {
	case "toml":
		{
			_, err.Err = toml.DecodeFile(file, &conf)
		}
	case "yaml":
		{
			var fdata []byte
			fdata, err.Err = ioutil.ReadFile(file)
			if !err.IsNull() {
				return
			}
			err.Err = yaml.Unmarshal(fdata, &conf)
		}
	case "json":
		{
			var fdata []byte
			fdata, err.Err = ioutil.ReadFile(file)
			if !err.IsNull() {
				return
			}
			err.Err = ffjson.Unmarshal(fdata, &conf)
		}
	}

	return
}

func ParseFiles(conf_type string, files...string) (err tree_lib.TreeError) {
	err.From = tree_lib.FROM_PARSE_FILE
	var combine_data []byte
	for _, f :=range files {
		var fdata []byte
		_, err.Err = ParseConfigFile(f)
		if !err.IsNull() {
			fmt.Println("error while reading ", f)
			fmt.Println("ignoring ", f)
			continue
		}
		fdata, err.Err = ioutil.ReadFile(f)
		if !err.IsNull() {
			return
		}
		combine_data = append(combine_data, fdata...)
		// Adding new line at the end of all files content
		combine_data = append(combine_data, []byte("\n")...)
	}

	switch conf_type {
	case "toml":
		{
			err.Err = toml.Unmarshal(combine_data, &GLOBAL_CONFIG)
		}
	case "yaml":
		{
			err.Err = yaml.Unmarshal(combine_data, &GLOBAL_CONFIG)
		}
	case "json":
		{
			err.Err = ffjson.Unmarshal(combine_data, &GLOBAL_CONFIG)
		}
	}
	return
}

func PathFiles(conf_type string, paths []string) ([]string, tree_lib.TreeError){
	var (
		err 		tree_lib.TreeError
		names 		[]string
		FileNames 	func(path string) (err tree_lib.TreeError)
	)
	err.From = tree_lib.FROM_PATH_FILES
	FileNames = func(path string) (err tree_lib.TreeError) {
		err.From = tree_lib.FROM_PATH_FILES
		files_in_dir, e := ioutil.ReadDir(path)
		err.Err = e
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return
		}

		for _, a := range files_in_dir {
			if !a.IsDir() {
				if filepath.Ext(a.Name())[0:] == conf_type {
					names = append(names, a.Name() + "." + conf_type)
				}
			} else {
				err = FileNames(string(path + "/" + a.Name()))
				if err != nil {
					tree_log.Error(err.From, err.Error())
					return
				}
			}
		}
		return
	}

	for _, a := range paths {
		if string([]rune(a)[len(a) - 1]) == "/" {
			a = a[:len(a)-1]
		}
		err = FileNames(a)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
			return nil, err
		}
	}
	return names, nil
}



func DBFromConfig() {
	var err tree_lib.TreeError
	err.From = tree_lib.FROM_DB_FROM_CONFIG

	for n, nf :=range GLOBAL_CONFIG.TreeNode {
		err = tree_db.SetNodeInfo(n, nf)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
		}
	}

	// After having All nodes information now we can set related things for every node
	for n, _ :=range GLOBAL_CONFIG.TreeNode {
		// Setting relations for every Node
		err = tree_db.SetRelations(n)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
		}

		// Setting Groups with node lists in Group database
		err = tree_db.AddNodeToHisGroups(n)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
		}

		// Setting Tags with node lists in Group database
		err = tree_db.AddNodeToHisTags(n)
		if !err.IsNull() {
			tree_log.Error(err.From, err.Error())
		}
	}

	// Setting Balancers
	for b, b_conf :=range GLOBAL_CONFIG.Balancer {
		var b_data []byte
		b_data, err.Err = ffjson.Marshal(b_conf)
		if !err.IsNull() {
			tree_log.Error(err.From, "Error encoding balancer config", b, " -> ", err.Error())
			continue
		}
		err = tree_db.Set(tree_db.DB_BALANCER, []byte(b), b_data)
		if !err.IsNull() {
			tree_log.Error(err.From, "Error setting balancer config", b, " -> ", err.Error())
		}
	}

	// Setting Registry
	for r, r_conf :=range GLOBAL_CONFIG.Registry {
		var r_data []byte
		r_data, err.Err = ffjson.Marshal(r_conf)
		if !err.IsNull() {
			tree_log.Error(err.From, "Error encoding registry config", r, " -> ", err.Error())
			continue
		}
		err = tree_db.Set(tree_db.DB_REGISTRY, []byte(r), r_data)
		if !err.IsNull() {
			tree_log.Error(err.From, "Error setting registry config", r, " -> ", err.Error())
		}
	}
}

func CompileConfig(cmd *cobra.Command, args []string) {
	var (
		files 			[]string
		conf_type 		string
		out_file		string
		paths 			[]string
		files_in_path	[]string
		err 			tree_lib.TreeError

	)
	err.From = tree_lib.FROM_COMPILE_CONFIG
	files, err.Err = cmd.Flags().GetStringSlice("files")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	conf_type, err.Err = cmd.Flags().GetString("type")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	out_file, err.Err = cmd.Flags().GetString("out")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	paths, err.Err = cmd.Flags().GetStringSlice("path")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	files_in_path, err.Err = PathFiles(conf_type, paths)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	for _, a := range files_in_path {
		files = append(files, a)
	}
	err = ParseFiles(conf_type, files...)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	DBFromConfig()
	err = tree_db.DumpDBPath(out_file)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
	// Deleting database dir from console part
	err.Err = os.RemoveAll(tree_db.DB_DIR)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
	}
}

func RestoreFromConfigDump(cmd *cobra.Command, args []string) {
	var (
		dump_file		string
		err 			tree_lib.TreeError
	)
	err.From = tree_lib.FROM_RESTORE_FROM_CONFIG_DUMP
	dump_file, err.Err = cmd.Flags().GetString("file")
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}

	err = tree_db.LoadFromDumpPath(dump_file)
	if !err.IsNull() {
		tree_log.Error(err.From, err.Error())
		return
	}
}