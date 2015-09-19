package tree_event

const (
	// Local events
	ON_PROGRAM_EXIT				=	"program_exit"

	// Networking events
	ON_PARENT_CONNECTED			=	"parent_connected"
	ON_PARENT_DISCONNECTED		=	"parent_disconnected"
	ON_CHILD_CONNECTED			=	"child_connected"
	ON_CHILD_DISCONNECTED		=	"child_disconnected"
	ON_API_CONNECTED			=	"api_disconnected"
	ON_API_DISCONNECTED			=	"api_disconnected"

	// API command events
	ON_API_COMMAND				=	"tree_api_command"
	ON_API_COMMAND_CALLBACK		=	"tree_api_command_callback"
)