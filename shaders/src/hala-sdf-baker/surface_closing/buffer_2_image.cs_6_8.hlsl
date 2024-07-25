#include "../baker/sdf_baker.hlsl"

[[vk::binding(0, 1)]]
StructuredBuffer<float4> _voxels_buffer;

[[vk::binding(1, 1)]]
RWTexture3D<float4> _voxels_texture_rw;

[numthreads(4, 4, 4)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z)
    return;

  const float4 voxel = _voxels_buffer[id3(id)];
  if (voxel.w != 0.0f)
    _voxels_texture_rw[id] = voxel;
}