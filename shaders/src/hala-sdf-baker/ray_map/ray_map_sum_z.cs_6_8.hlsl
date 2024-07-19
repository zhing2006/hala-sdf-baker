#include "../baker/baker.hlsl"
#include "ray_map.hlsl"

[[vk::binding(0, 1)]]
RWTexture3D<float4> _ray_map_rw;

[numthreads(8, 8, 1)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y)
    return;

  // Sum the intersection counts in the z-direction.
  for (int t = _dimensions.z - 2; t >= 0; t--) {
    float count = _ray_map_rw[int3(id.x, id.y, t + 1)].z;
    _ray_map_rw[int3(id.x, id.y, t)] += float4(0, 0, count, count != 0 ? 1 : 0);
  }
}