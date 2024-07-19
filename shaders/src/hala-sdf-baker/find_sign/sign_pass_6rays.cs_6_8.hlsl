#include "../baker/baker.hlsl"

[[vk::binding(0, 1)]]
Texture3D<float4> _ray_map;

[[vk::binding(1, 1)]]
RWTexture3D<float> _sign_map_rw;

[numthreads(4, 4, 4)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z)
    return;

  const float4 selfRayMap = _ray_map[id.xyz];
  _sign_map_rw[id.xyz] = (
    selfRayMap.x + // right side intersection.
    selfRayMap.y + // bake side intersection.
    selfRayMap.z + // top side intersection.
    (selfRayMap.x - _ray_map[int3(0, id.y, id.z)].x) +  // left side intersection(negative).
    (selfRayMap.y - _ray_map[int3(id.x, 0, id.z)].y) +  // front side intersection(negative).
    (selfRayMap.z - _ray_map[int3(id.x, id.y, 0)].z)    // bottom side intersection(negative).
  );
}