#include "../baker/sdf_baker.hlsl"
#include "ray_map.hlsl"

[[vk::binding(0, 1)]]
RWTexture3D<float4> _ray_map_rw;

[numthreads(1, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.y >= _dimensions.y || id.z >= _dimensions.z)
    return;

  // Sum the intersection counts in the x-direction.
  // Each voxel contains the intersections count(-axis is negative, +axis is positive).
  for (int t = _dimensions.x - 2; t >= 0; t--) {
    float count = _ray_map_rw[int3(t + 1, id.y, id.z)].x;
    _ray_map_rw[int3(t, id.y, id.z)] += float4(count, 0, 0, count != 0 ? 1 : 0);
  }
}