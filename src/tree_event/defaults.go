package tree_event

const (
	// Local events
	ON_PROGRAM_EXIT				=	"program_exit"

		// Docker Events
		ON_DOCKER_INIT				=	"docker_init"
		ON_DOCKER_END				=	"docker_end"
		ON_DOCKER_CONTAINER_START	=	"docker_container_start"
		ON_DOCKER_CONTAINER_STOP	=	"docker_container_stop"
		ON_DOCKER_IMAGE_CREATE		=	"docker_image_create"
		ON_DOCKER_IMAGE_DELETE		=	"docker_image_delete"

		// Balancer Events
		ON_BALANCER_INIT			=	"balancer_init"
		ON_BALANCER_END				=	"balancer_end"
		ON_BALANCER_SERVICE_START	=	"balancer_service_start"
		ON_BALANCER_SERVICE_STOP	=	"balancer_service_stop"

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

	// DB events
	ON_UPDATE_NODE_INFO		=	"update_node_info"
)