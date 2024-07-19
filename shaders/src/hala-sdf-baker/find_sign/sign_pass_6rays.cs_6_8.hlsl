#include "../baker/sdf_baker.hlsl"

[[vk::binding(0, 1)]]
Texture3D<float4> _ray_map;

[[vk::binding(1, 1)]]
RWTexture3D<float> _sign_map_rw;

[numthreads(4, 4, 4)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z)
    return;

  const float4 self_ray_map = _ray_map[id.xyz];
  _sign_map_rw[id.xyz] = (
    self_ray_map.x + // right side intersection.
    self_ray_map.y + // bake side intersection.
    self_ray_map.z + // top side intersection.
    (self_ray_map.x - _ray_map[int3(0, id.y, id.z)].x) +  // left side intersection(negative).
    (self_ray_map.y - _ray_map[int3(id.x, 0, id.z)].y) +  // front side intersection(negative).
    (self_ray_map.z - _ray_map[int3(id.x, id.y, 0)].z)    // bottom side intersection(negative).
  );
}