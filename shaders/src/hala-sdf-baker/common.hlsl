#define MAX_CAMERAS 8
#define MAX_LIGHTS 16
#define INVALID_INDEX 0xFFFFFFFF

struct Camera {
  float3 position;  // camera position
  float3 right;     // camera right vector
  float3 up;        // camera up vector
  float3 forward;   // camera forward vector
  float yfov;       // vertical field of view
  float focal_distance_or_xmag; // focal distance for perspective camera and xmag for orthographic camera
  float aperture_or_ymag; // aperture size for perspective camera and ymag for orthographic camera
  uint type;      // 0 - perspective camera, 1 - orthographic camera
};

struct Light {
  float3 intensity;
  // For point light, position is the position.
  // For directional light, position is unused.
  // For spot light, quad light and sphere light, position is the position.
  float3 position;
  // For point light, u is unused.
  // For directional light and spot light, u is the direction.
  // For quad light, u is the right direction and length.
  // For sphere light, u is unused.
  float3 u;
  // For point light, v is unused.
  // For directional light, v.x is the cosine of the cone angle.
  // For spot light, v.x is the cosine of the inner cone angle, v.y is the cosine of the outer cone angle.
  // For quad light, v is the up direction and length.
  // For sphere light, v is unused.
  float3 v;
  // For point light, directional light, spot light and quad light, radius is unused.
  // For sphere light, radius is the radius.
  float radius;
  // For point light, directional light and spot light, area is unused.
  // For quad light and sphere light, area is the area.
  float area;
  // light type: 0 - point, 1 - directional, 2 - spot, 3 - quad, 4 - sphere
  int type;
};

struct Medium {
  float3 color;
  float density;
  float anisotropy;
  uint type;
  float padding0;
  float padding1;
};

struct Material {
  Medium medium;

  [[vk::offset(32)]]
  float3 base_color;
  float opacity;

  float3 emission;
  float anisotropic;

  float metallic;
  float roughness;
  float subsurface;
  float specular_tint;

  float sheen;
  float sheen_tint;
  float clearcoat;
  float clearcoat_roughness;

  float3 clearcoat_tint;
  float specular_transmission;

  float ior;
  float ax;
  float ay;
  uint base_color_map_index;

  uint normal_map_index;
  uint metallic_roughness_map_index;
  uint emission_map_index;
  uint type;
};

struct PushConstants {
  uint object_index;
  uint material_index;
  uint primitive_index;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 0)]]
cbuffer GlobalUniformBuffer {
  float4x4 v_mtx;   // The view matrix
  float4x4 p_mtx;   // The projection matrix
  float4x4 vp_mtx;  // The view-projection matrix
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

[[vk::binding(0, 2)]]
Texture2D<float4> g_textures[];

[[vk::binding(1, 2)]]
SamplerState g_samplers[];