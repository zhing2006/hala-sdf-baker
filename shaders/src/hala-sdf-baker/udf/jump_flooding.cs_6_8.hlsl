#include "../baker/udf_baker.hlsl"

struct PushConstants {
  int offset;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 1)]]
StructuredBuffer<uint> _jump_buffer;

[[vk::binding(1, 1)]]
RWTexture3D<float> _distance_texture_rw;

[[vk::binding(2, 1)]]
RWStructuredBuffer<uint> _jump_buffer_rw;

void jump_sample(int3 center_coord, int3 offset, inout float best_distance, inout int best_index) {
  int3 sample_coord = center_coord + offset;
  if (
    sample_coord.x < 0 || sample_coord.y < 0 || sample_coord.z < 0 ||
    sample_coord.x >= _dimensions.x || sample_coord.y >= _dimensions.y || sample_coord.z >= _dimensions.z
  ) {
    return;
  }
  uint voxel_sample_index = _jump_buffer[id3(sample_coord)];
  int3 voxel_sample_coord = unpack_id3(voxel_sample_index);
  float voxel_sample_distance = _distance_texture_rw[voxel_sample_coord];
  float distance = length(float3(center_coord) / _max_dimension - float3(voxel_sample_coord) / _max_dimension) + voxel_sample_distance;
  if (voxel_sample_index != 0xFFFFFFFF && distance < best_distance) {
    best_distance = distance;
    best_index = voxel_sample_index;
  }
}

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }

  // if (id.x == 0 && id.y == 0 && id.z == 0) {
  //   printf("Offset: %d\n", g_push_constants.offset);
  // }

  float best_distance = _initial_distance;
  int best_index = 0xFFFFFFFF;

  [unroll(3)]
  for (int z = -1; z <= 1; ++z)
    [unroll(3)]
    for (int y = -1; y <= 1; ++y)
      [unroll(3)]
      for (int x = -1; x <= 1; ++x)
        jump_sample(id, int3(x, y, z) * g_push_constants.offset, best_distance, best_index);

  if (best_index != 0xFFFFFFFF) {
    _distance_texture_rw[int3(id.x, id.y, id.z)] = best_distance;
    _jump_buffer_rw[id3(id.x, id.y, id.z)] = best_index;
  }
}