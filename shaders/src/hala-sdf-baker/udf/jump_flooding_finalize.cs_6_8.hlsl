#include "../baker/udf_baker.hlsl"

[[vk::binding(0, 1)]]
StructuredBuffer<uint> _jump_buffer;

[[vk::binding(1, 1)]]
RWTexture3D<float> _distance_texture_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }

  const uint voxel_index = id3(id.x, id.y, id.z);

  int closest_seed_voxel_index = _jump_buffer[voxel_index];
  int3 closest_seed_voxel_coord = unpack_id3(closest_seed_voxel_index);
  float distance_to_closest_seed_voxel = length(get_position(int3(id.x, id.y, id.z)) - get_position(closest_seed_voxel_coord)) * _voxel_size;
  float distance_of_closest_seed_voxel_to_surface = _distance_texture_rw[closest_seed_voxel_coord];

  _distance_texture_rw[int3(id.x, id.y, id.z)] = distance_to_closest_seed_voxel + distance_of_closest_seed_voxel_to_surface;
}