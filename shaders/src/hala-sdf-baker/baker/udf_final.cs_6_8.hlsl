#include "udf_baker.hlsl"

[[vk::binding(0, 1)]]
RWTexture3D<uint> _distance_texture_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }

  const int3 uvw = int3(id.x, id.y, id.z);
  const uint distance = _distance_texture_rw[uvw];
  _distance_texture_rw[uvw] = float_unflip(distance);
}