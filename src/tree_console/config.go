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
)

const (
	log_from_config	=	"From config functionality"
)

type TreeScaleConf struct {
	SSH				map[string]SSHConfig				`toml:"ssh" json:"ssh" yaml:"ssh"`
	TreeNode		map[string]node_info.NodeInfo		`toml:"tree_node" json:"tree_node" yaml:"tree_node"`
	// TODO: Add docker registry config here
	// TODO: Add balancer config here
}

var (
	GLOBAL_CONFIG	TreeScaleConf
)

func ParseConfigFile(file string) (conf TreeScaleConf, err error) {
	switch strings.Replace(filepath.Ext(file), ".", "", 1) {
	case "toml":
		{
			_, err = toml.DecodeFile(file, &conf)
		}
	case "yaml":
		{
			var fdata []byte
			fdata, err = ioutil.ReadFile(file)
			if err != nil {
				return
			}
			err = yaml.Unmarshal(fdata, &conf)
		}
	case "json":
		{
			var fdata []byte
			fdata, err = ioutil.ReadFile(file)
			if err != nil {
				return
			}
			err = ffjson.Unmarshal(fdata, &conf)
		}
	}

	return
}

func ParseFiles(conf_type string, files...string) (err error) {
	var combine_data []byte
	for _, f :=range files {
		var fdata []byte
		fdata, err = ioutil.ReadFile(f)
		if err != nil {
			return
		}
		combine_data = append(combine_data, fdata...)
	}

	switch conf_type {
	case "toml":
		{
			err = toml.Unmarshal(combine_data, &GLOBAL_CONFIG)
		}
	case "yaml":
		{
			err = yaml.Unmarshal(combine_data, &GLOBAL_CONFIG)
		}
	case "json":
		{
			err = ffjson.Unmarshal(combine_data, &GLOBAL_CONFIG)
		}
	}
	return
}

func DBFromConfig() {
	for n, nf :=range GLOBAL_CONFIG.TreeNode {
		err := tree_db.SetNodeInfo(n, nf)
		if err != nil {
			tree_log.Error(log_from_config, err.Error())
		}
	}
}

func CompileConfig(cmd *cobra.Command, args []string) {
	files, err := cmd.Flags().GetStringSlice("files")
	if err != nil {
		tree_log.Error(log_from_config, err.Error())
		return
	}

	conf_type, err := cmd.Flags().GetString("type")
	if err != nil {
		tree_log.Error(log_from_config, err.Error())
		return
	}

	out_file, err := cmd.Flags().GetString("out")
	if err != nil {
		tree_log.Error(log_from_config, err.Error())
		return
	}


	err = ParseFiles(conf_type, files...)
	if err != nil {
		tree_log.Error(log_from_config, err.Error())
		return
	}
	DBFromConfig()
	err = tree_db.DumpDBPath(out_file)
	if err != nil {
		tree_log.Error(log_from_config, err.Error())
		return
	}
	// Deleting database dir from console part
	err = os.RemoveAll(tree_db.DB_DIR)
	if err != nil {
		tree_log.Error(log_from_config, err.Error())
	}
}

func RestoreFromConfigDump(cmd *cobra.Command, args []string) {
	dump_file, err := cmd.Flags().GetString("file")
	if err != nil {
		tree_log.Error(log_from_config, err.Error())
		return
	}

	err = tree_db.LoadFromDumpPath(dump_file)
	if err != nil {
		tree_log.Error(log_from_config, err.Error())
		return
	}
}