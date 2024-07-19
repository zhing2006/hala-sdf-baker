#define AABB_EPS 1e-5

struct PushConstants {
  uint current_axis;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 0)]]
cbuffer GlobalUniformBuffer {
  uint3 _dimensions;
  uint _max_dimension;
  uint _upper_bound_count;
  uint _num_triangles;
  float _max_extent;
  float _padding0;
  float3 _min_bounds_extended;
  float _padding1;
  float3 _max_bounds_extended;
  float _padding2;
};

[[vk::binding(0, 1)]]
StructuredBuffer<float4> _vertices_buffer;

[[vk::binding(1, 1)]]
StructuredBuffer<int> _coord_flip_buffer;

[[vk::binding(2, 1)]]
StructuredBuffer<float4> _aabb_buffer;

[[vk::binding(3, 1)]]
RWStructuredBuffer<float4> _voxels_buffer_rw;

[[vk::binding(4, 1)]]
RWStructuredBuffer<uint> _counter_buffer_rw;

[[vk::binding(5, 1)]]
RWStructuredBuffer<uint> _triangle_ids_buffer_rw;

struct ToFragment {
  float4 position: SV_Position;
  [[vk::location(0)]] uint triangle_id: TEXCOORD0;
};

inline uint id3(int i, int j, int k) {
  return (uint)(i + _dimensions[0] * j + _dimensions[0] * _dimensions[1] * k);
}

inline uint id3(int3 coord) {
  return id3(coord.x, coord.y, coord.z);
}

float2 get_custom_screen_params() {
  if (g_push_constants.current_axis == 1) {
    return float2(_dimensions[2], _dimensions[0]);
  } else if (g_push_constants.current_axis == 2) {
    return float2(_dimensions[1], _dimensions[2]);
  } else {
    return float2(_dimensions[0], _dimensions[1]);
  }
}

void screen_to_uvw(inout float4 screen_position, float2 screen_params) {
  // For reversed Z.
  screen_position.z = 1.0f - screen_position.z;
  screen_position.xy = screen_position.xy / screen_params;
  // For UV starts at TOP.
  screen_position.y = 1.0f - screen_position.y;
}

void cull_with_aabb(float4 screen_position, int triangle_id) {
  const float2 ndc_pos = screen_position.xy * 2.0f - 1.0f;
  const float4 aabb = _aabb_buffer[triangle_id];
  if (
    ndc_pos.x < aabb.x - AABB_EPS ||
    ndc_pos.y < aabb.y - AABB_EPS ||
    ndc_pos.x > aabb.z + AABB_EPS ||
    ndc_pos.y > aabb.w + AABB_EPS
  ) {
    discard;
  }
}

void compute_coord_and_depth_step(
  float2 screen_params,
  float4 screen_position,
  out int3 voxel_coord
#ifdef USE_CONSERVATIVE_RASTERIZATION
  ,
  out int3 depth_step,
  out bool can_step_backward,
  out bool can_step_forward
#endif
) {
#ifdef USE_CONSERVATIVE_RASTERIZATION
  // we're conservative about how we share triangle data across neighbouring cells, to fix visible artefacts
  can_step_forward = true;
  can_step_backward = true;
#endif
  if (g_push_constants.current_axis == 1) {
    voxel_coord = (screen_position.xyz * float3(screen_params, _dimensions[1]));
    voxel_coord.xyz = voxel_coord.yzx;
#ifdef USE_CONSERVATIVE_RASTERIZATION
    depth_step = int3(0, 1, 0);
    if (voxel_coord.y <= 0) {
      can_step_backward = false;
    }
    if (voxel_coord.y >= _dimensions[1] - 1) {
      can_step_forward = false;
    }
#endif
  } else if (g_push_constants.current_axis == 2) {
    voxel_coord = (screen_position.xyz * float3(screen_params, _dimensions[0]));
    voxel_coord.xyz = voxel_coord.zxy;
#ifdef USE_CONSERVATIVE_RASTERIZATION
    depth_step = int3(1, 0, 0);
    if (voxel_coord.x <= 0) {
      can_step_backward = false;
    }
    if (voxel_coord.x >= _dimensions[0] - 1) {
      can_step_forward = false;
    }
#endif
  } else {
    voxel_coord = (screen_position.xyz * float3(screen_params, _dimensions[2]));
#ifdef USE_CONSERVATIVE_RASTERIZATION
    depth_step = int3(0, 0, 1);
    if (voxel_coord.z <= 0) {
      can_step_backward = false;
    }
    if (voxel_coord.z >= _dimensions[2] - 1) {
      can_step_forward = false;
    }
#endif
  }
}

void get_voxel_coordinates(
  float4 screen_position,
  uint triangle_id,
  out int3 voxel_coord
#ifdef USE_CONSERVATIVE_RASTERIZATION
  ,
  out int3 depth_step,
  out bool can_step_backward,
  out bool can_step_forward
#endif
) {
  const float2 screen_params = get_custom_screen_params();
  screen_to_uvw(screen_position, screen_params);
  cull_with_aabb(screen_position, triangle_id);
  compute_coord_and_depth_step(
    screen_params,
    screen_position,
    voxel_coord
#ifdef USE_CONSERVATIVE_RASTERIZATION
    ,
    depth_step,
    can_step_backward,
    can_step_forward
#endif
  );
}