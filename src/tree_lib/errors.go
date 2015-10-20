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
	FROM_SET = "From Set"
	FROM_GET = "From Get"
	FROM_FOREACH = "From ForEach"
	FROM_LIST_NODE_INFOS = "from ListNodeInfos"
	FROM_LIST_NODE_NAMES = "From ListNodeNames"
	FROM_GET_NODE_INFO = "From GetNodeInfo"
	FROM_SET_NODE_INFO = "From SetNodeInfo"
	FROM_SET_RELATIONS = "From SetRelations"
	FROM_GET_RELATIONS = "From GetRelations"
	FROM_GET_GROUP_NODES = "From GetGroupNodes"
	FROM_GROUP_ADD_NODE = "From GroupAddNode"
	FROM_ADD_NODE_TO_HIS_GROUPS = "From AddNodeToHisGroups"
	FROM_GET_NODES_BY_TAG_NAME = "From GetNodesByTagName"
	FROM_TAG_ADD_NODE = "From TagAddNode"
	FROM_ADD_NODE_TO_HIS_TAGS = "From AddNodeToHisTags"
	FROM_GET_PARENT_INFO = "From GetParentInfo"
	FROM_DB_FROM_CONFIG = "From DBFromConfig"
	FROM_COMPILE_CONFIG = "From CompileConfig"
	FROM_RESTORE_FROM_CONFIG_DUMP = "From RestoreFromConfigDump"
	FROM_HANDLE_NODE_COMMAND = "From HandleNodeCommand"
	FROM_SSH_CONNECT = "From SSH Connect"
	FROM_SSH_DISCONNECT = "Form SSH Disconnect"
	FROM_SSH_EXEC = "From SSH Exec"
	FROM_SSH_COPY_FILE = "From SSH CopyFile"
	FROM_HANDLE_API_COMMAND = "From HandleApiCommand"
	FROM_CONTAINER_COMMANDS = "From ContainerCommands"
	FROM_SSL_EXCEPTIONS = "From SSLExceptions"
	FROM_INIT_DOCKER_CLIENT = "From InitDockerClient"
	FROM_TRIGGER_INIT_EVENT = "From TriggerInitEvent"
	FROM_START_EVENT_LISTENER = "From StartEventListener"
	FROM_CALL_EVENT = "From CallEvent"
	FROM_TRIGGER_FROM_DATA = "From TriggerFromData"
	FROM_PATH_FROM_MESSAGE = "From PathFromMessage"
	FROM_GROUP_PATH = "From GroupPath"
	FROM_NODE_PATH = "From NodePath"
	FROM_TAG_PATH = "From TagPath"
	FROM_GET_PATH = "From GetPath"
	ROM_COPY_FILE = "From CopyFile"
	FROM_READ_MESSAGE = "From ReadMessage"
	FROM_READ_JSON = "From ReadJson"
	FROM_SEND_JSON = "From SendJson"
	FROM_SET_PARENT = "From SetParrent"
	FROM_SET_CURRENT_NODE = "From SetCurrentNode"
	FROM_NODE_INIT = "From NodeInit"
	FROM_INIT_BALANCER = "From InitBalancer"
	FROM_LISTEN_PARENT = "From ListenParent"
	FROM_HANDLE_API_OR_PARENT_CONNECTION = "From handle_api_or_parent_connection"
	FROM_CHILD_CONNECT = "From ChildConnect"
	FROM_HANDLE_MESSAGE = "From handle_message"
	FROM_SEND_TO_NAMES = "From SendToNames"
	FROM_SEND_TO_CONN = "From SendToConn"
	FROM_TCP_CONNECT = "From TCPConnect"
	FROM_NETWORK_EMIT = "From NetworkEmit"
	FROM_API_EMIT = "From ApiEmit"
	FROM_EMIT_TO_API = "From EmitToApi"
	FROM_COPY_FILE = "From CopyFile"
	FROM_UPDATE_NODE_CHANGE = "From UpdateNodeChange"
	FROM_SET_PATH_VALUES = "From SetPathValues"
	FROM_GET_NODE_VALUE = "From GetNodeValue"
	FROM_MERGE = "From merge"
	FROM_LIST_INFOS = "From ListInfos"
	FROM_HANDLE_CONTAINER_COMMAND = "From HandleContainerCommand"
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