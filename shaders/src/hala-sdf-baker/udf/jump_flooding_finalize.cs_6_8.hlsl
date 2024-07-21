#include "../baker/udf_baker.hlsl"


[[vk::binding(0, 1)]]
Texture3D<float> _distance_texture;

[[vk::binding(1, 1)]]
RWStructuredBuffer<float> _distance_buffer_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }

  const uint voxel_index = id3(id.x, id.y, id.z);
  _distance_buffer_rw[voxel_index] = _distance_texture[int3(id.x, id.y, id.z)];
}