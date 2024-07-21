#include "../baker/udf_baker.hlsl"

#define SQRT_3 1.73205081

[[vk::binding(0, 1)]]
Texture3D<float> _distance_texture;

[[vk::binding(1, 1)]]
RWStructuredBuffer<uint> _jump_buffer_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }

  const float distance = _distance_texture[int3(id.x, id.y, id.z)];
  const uint voxel_index = id3(id.x, id.y, id.z);

  // Even though more voxels are initialized, we want to treat as seeds only the ones
  // within one voxel of the surface. Otherwise the distance estimate is not very smooth
  // as it sees the chunky bounding boxes of bigger triangles.
  _jump_buffer_rw[voxel_index] = abs(distance) > SQRT_3 ? 0xFFFFFFFF : voxel_index;
}