#include "../baker/sdf_baker.hlsl"

struct PushConstants {
  float threshold;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 1)]]
Texture3D<float> _sign_map;

[[vk::binding(1, 1)]]
RWTexture3D<float4> _voxels_texture_rw;

[numthreads(4, 4, 4)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z)
    return;

  const float self_sign_score = _sign_map[id.xyz] - g_push_constants.threshold;
  if (abs(self_sign_score / g_push_constants.threshold) < 0.1f) {
    if (self_sign_score * (_sign_map[id.xyz + uint3(1, 0, 0)] - g_push_constants.threshold) < 0) {
      const uint3 write_coord = id.xyz + (self_sign_score < 0 ? uint3(1, 0, 0) : uint3(0, 0, 0));
      _voxels_texture_rw[write_coord.xyz] = float4((float3(write_coord) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension, 1.0f);
    }
    if (self_sign_score * (_sign_map[id.xyz + uint3(0, 1, 0)] - g_push_constants.threshold) < 0) {
      const uint3 write_coord = id.xyz + (self_sign_score < 0 ? uint3(0, 1, 0) : uint3(0, 0, 0));
      _voxels_texture_rw[write_coord.xyz] = float4((float3(write_coord) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension, 1.0f);
    }
    if (self_sign_score * (_sign_map[id.xyz + uint3(0, 0, 1)] - g_push_constants.threshold) < 0) {
      const uint3 write_coord = id.xyz + (self_sign_score < 0 ? uint3(0, 0, 1) : uint3(0, 0, 0));
      _voxels_texture_rw[write_coord.xyz] = float4((float3(write_coord) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension, 1.0f);
    }
  }
}