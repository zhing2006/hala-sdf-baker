#include "../baker/udf_baker.hlsl"

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
  const uint closest_seed_voxel_index = _jump_buffer[voxel_index];
  if (closest_seed_voxel_index == 0xFFFFFFFF) {
    return;
  }
  const int3 closest_seed_voxel_id = unpack_id3(closest_seed_voxel_index);
  const float3 closest_seed_voxel_coord = float3(closest_seed_voxel_id) / _max_dimension;
  const float distance_to_closest_seed_voxel = length(float3(id.x, id.y, id.z) / _max_dimension - closest_seed_voxel_coord);
  const float distance_of_closest_seed_voxel_to_surface = _distance_texture_rw[closest_seed_voxel_id];

  _distance_texture_rw[int3(id.x, id.y, id.z)] = distance_to_closest_seed_voxel + distance_of_closest_seed_voxel_to_surface;
  _distance_buffer_rw[voxel_index] = distance_to_closest_seed_voxel + distance_of_closest_seed_voxel_to_surface;
}