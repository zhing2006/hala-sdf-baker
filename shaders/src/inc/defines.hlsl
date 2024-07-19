#define MAX_CAMERAS 8
#define MAX_LIGHTS 16
#define INVALID_INDEX 0xFFFFFFFF
#define DIV_UP(a, b) (((a) + (b) - 1) / (b))

#ifdef USE_MESH_SHADER
#define TASK_SHADER_GROUP_SIZE 32
#define MESH_SHADER_GROUP_SIZE 64
#endif

#define ERROR_COLOR float4(1, 0, 1, 1)
