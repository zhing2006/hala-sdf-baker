#include "../baker/sdf_baker.hlsl"
#include "ray_map.hlsl"

[[vk::binding(0, 1)]]
RWTexture3D<float4> _ray_map_rw;

[numthreads(8, 1, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.z >= _dimensions.z)
    return;

  // Sum the intersection counts in the y-direction.
  for (int t = _dimensions.y - 2; t >= 0; t--) {
    float count = _ray_map_rw[int3(id.x, t + 1, id.z)].y;
    _ray_map_rw[int3(id.x, t, id.z)] += float4(0, count, 0, count != 0 ? 1 : 0);
  }
}