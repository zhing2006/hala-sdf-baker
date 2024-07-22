#include "../baker/udf_baker.hlsl"

struct PushConstants {
  float offset;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 1)]]
RWTexture3D<float> _distance_texture_rw;

[[vk::binding(1, 1)]]
RWStructuredBuffer<float> _distance_buffer_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }

  const uint voxel_index = id3(id.x, id.y, id.z);
  const float distance = _distance_texture_rw[int3(id.x, id.y, id.z)] + g_push_constants.offset;
  _distance_texture_rw[int3(id.x, id.y, id.z)] = distance;
  _distance_buffer_rw[voxel_index] = distance;
}