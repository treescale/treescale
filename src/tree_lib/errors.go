package tree_lib


const (
	FROM_INIT = "From Init"
	FROM_WRITE = "From Write"
	FROM_SEND_COMMAND = "From SendCommand"
	FROM_API_INIT = "From Api_Init"
	FROM_HANDLE_EXEC_COMMAND = "From HandleExecCommand"
	FROM_ADD_SERVICE = "From AddService"
	FROM_DROP_SERVICE = "From DropService"
	FROM_SUBSCRIBE_EVENTS = "From SubscribeEvents"
	FROM_HANDLE_API_EXEC = "From HandleApiExec"
	FROM_BUILD_TREE = "From BuidTree"
	FROM_INSTALL_DOCKER = "From InstallDocker"
	FROM_INSTALL_TREESCALE = "From InstallTreeScale"
	FROM_PARSE_CONFIG_FILE = "From ParseConfigFile"
	FROM_PARSE_FILE = "From ParseFile"
	FROM_PATH_FILES = "From PathFiles"
)

const (
	SYNTAX_ERROR = 1
)
type TreeError struct{
	Err			error
	Code		int			`json:"code" toml:"code" yaml:"code"`
	From		string
}

func (e *TreeError) Error() (string) {
	return e.Err.Error()
}

func (e *TreeError) IsNull() bool {
	return e.Err == nil
}