#include "defines.hlsl"
#include "types.hlsl"

struct PushConstants {
  uint object_index;
  uint material_index;
  uint primitive_index;
#ifdef USE_MESH_SHADER
  uint meshlet_count;
  uint dispatch_size_x;
#endif
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 0)]]
cbuffer GlobalUniformBuffer {
  float4x4 v_mtx;       // The view matrix
  float4x4 p_mtx;       // The projection matrix
  float4x4 vp_mtx;      // The view-projection matrix
};

[[vk::binding(1, 0)]]
cbuffer CameraBuffer {
  Camera cameras[MAX_CAMERAS];
};

[[vk::binding(2, 0)]]
cbuffer LightBuffer {
  Light lights[MAX_LIGHTS];
};

[[vk::binding(0, 1)]]
ConstantBuffer<Material> g_materials[];

struct ObjectUniformBuffer {
  float4x4 m_mtx;     // The model matrix
  float4x4 i_m_mtx;   // The inverse model matrix
  float4x4 mv_mtx;    // The model-view matrix
  float4x4 t_mv_mtx;  // The transposed model-view matrix
  float4x4 it_mv_mtx; // The inverse transposed model-view matrix
  float4x4 mvp_mtx;   // The model-view-projection matrix
};

[[vk::binding(1, 1)]]
ConstantBuffer<ObjectUniformBuffer> g_per_object_data[];

struct Vertex {
  float position_x;
  float position_y;
  float position_z;
  float normal_x;
  float normal_y;
  float normal_z;
  float tangent_x;
  float tangent_y;
  float tangent_z;
  float tex_coord_x;
  float tex_coord_y;
};

[[vk::binding(2, 1)]]
StructuredBuffer<Vertex> g_vertices[];

[[vk::binding(3, 1)]]
StructuredBuffer<uint> g_indices[];

struct Meshlet {
  float4 bound_sphere;  // center, radius
  float3 cone_apex;
  float cone_cutoff;
  float3 cone_axis;
  uint num_of_vertices;
  uint num_of_primitives;
  uint offset_of_vertices;
  uint offset_of_primitives;
  float padding;
};

[[vk::binding(4, 1)]]
StructuredBuffer<Meshlet> g_meshlets[];

[[vk::binding(5, 1)]]
StructuredBuffer<uint> g_unique_vertices[];

[[vk::binding(6, 1)]]
ByteAddressBuffer g_unique_primitives[];

[[vk::binding(0, 2)]]
Texture2D<float4> g_textures[];

[[vk::binding(1, 2)]]
SamplerState g_samplers[];

#ifdef USE_MESH_SHADER
struct MeshShaderPayLoad {
  uint task_group_id;
  bool is_visibles[TASK_SHADER_GROUP_SIZE];
};
#endif