#include "defines.hlsl"
#include "types.hlsl"

struct PushConstants {
  float center[3];
  float extents[3];
  float color[4];
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 0)]]
cbuffer GlobalUniformBuffer {
  float4x4 _m_mtx;     // The model matrix
  float4x4 _i_m_mtx;   // The inverse model matrix
  float4x4 _vp_mtx;    // The view-projection matrix
  float4x4 _mvp_mtx;   // The model-view-projection matrix
  float3 _camera_position;
  float _offset;
  uint3 _dimensions;
  float _inv_resolution;
};

[[vk::combinedImageSampler]]
[[vk::binding(1, 0)]]
Texture3D<float4> g_texture;

[[vk::combinedImageSampler]]
[[vk::binding(1, 0)]]
SamplerState g_sampler;

struct ToFragment {
  float4 position: SV_Position;
  [[vk::location(0)]] float3 position_ws: TEXCOORD0;
  [[vk::location(1)]] float4 color: COLOR0;
};

struct FragmentOutput {
  [[vk::location(0)]] float4 color: SV_Target0;
  [[vk::location(1)]] float depth : SV_Depth;
};

static const float3 g_vertices[36] = {
  // XZ planes.
  float3(-1.0,-1.0,-1.0), float3(1.0,-1.0,-1.0), float3(-1.0,-1.0,1.0),
  float3(-1.0,-1.0, 1.0), float3(1.0,-1.0,-1.0), float3( 1.0,-1.0,1.0),
  float3(-1.0, 1.0,-1.0), float3(-1.0, 1.0,1.0), float3(1.0, 1.0,-1.0),
  float3(-1.0, 1.0, 1.0), float3( 1.0, 1.0,1.0), float3(1.0, 1.0,-1.0),
  // XY planes.
  float3(-1.0,-1.0, 1.0), float3(1.0,-1.0, 1.0), float3(-1.0,1.0, 1.0),
  float3(-1.0, 1.0, 1.0), float3(1.0,-1.0, 1.0), float3( 1.0,1.0, 1.0),
  float3(-1.0,-1.0,-1.0), float3(-1.0,1.0,-1.0), float3(1.0,-1.0,-1.0),
  float3(-1.0, 1.0,-1.0), float3( 1.0,1.0,-1.0), float3(1.0,-1.0,-1.0),
  // YZ planes.
  float3( 1.0,-1.0,-1.0), float3( 1.0,1.0,-1.0), float3( 1.0,-1.0,1.0),
  float3( 1.0,-1.0, 1.0), float3( 1.0,1.0,-1.0), float3( 1.0, 1.0,1.0),
  float3(-1.0,-1.0,-1.0), float3(-1.0,-1.0,1.0), float3(-1.0,1.0,-1.0),
  float3(-1.0,-1.0, 1.0), float3(-1.0, 1.0,1.0), float3(-1.0,1.0,-1.0),
};

// Ray origin is "ray_origin", ray direction is "ray_direction"
// Returns "t" along the ray of min, max intersection, or (-1,-1) if no intersections are found.
// https://iquilezles.org/www/articles/intersectors/intersectors.htm
float2 ray_box_intersection(float3 ray_origin, float3 ray_direction, float3 box_size) {
  float3 inv_ray_dir = 1.0 / ray_direction;
  float3 neg_origin_times_inv_dir = inv_ray_dir * ray_origin;
  float3 abs_inv_ray_dir_times_box_size = abs(inv_ray_dir) * box_size;
  float3 near_point_times = -neg_origin_times_inv_dir - abs_inv_ray_dir_times_box_size;
  float3 far_point_times = -neg_origin_times_inv_dir + abs_inv_ray_dir_times_box_size;
  float nearest_t = max(max(near_point_times.x, near_point_times.y), near_point_times.z);
  float farthest_t = min(min(far_point_times.x, far_point_times.y), far_point_times.z);
  if (nearest_t > farthest_t || farthest_t < 0.0) return -1; // no intersection
  return float2(nearest_t, farthest_t);
}
