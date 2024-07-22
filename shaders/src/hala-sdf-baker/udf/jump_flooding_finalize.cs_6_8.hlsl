#include "../baker/udf_baker.hlsl"

struct PushConstants {
  float offset;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 1)]]
StructuredBuffer<uint> _jump_buffer;

[[vk::binding(1, 1)]]
RWTexture3D<float> _distance_texture_rw;

[[vk::binding(2, 1)]]
RWStructuredBuffer<float> _distance_buffer_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }

  const uint voxel_index = id3(id.x, id.y, id.z);

  const uint cloest_voxel_index = _jump_buffer[voxel_index];
  const int3 cloest_voxel_coord = unpack_id3(cloest_voxel_index);
  const float cloest_voxel_distance = _distance_texture_rw[cloest_voxel_coord];

  const float distance_to_cloest_voxel = length(float3(id) / _max_dimension - float3(cloest_voxel_coord) / _max_dimension);

  _distance_texture_rw[int3(id.x, id.y, id.z)] = cloest_voxel_distance + distance_to_cloest_voxel + g_push_constants.offset;
  _distance_buffer_rw[voxel_index] = cloest_voxel_distance + distance_to_cloest_voxel + g_push_constants.offset;
}