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
  // From right to current voxel face intersection count.
  const float right_side_intersection = self_ray_map.x;
  // From back to current voxel face intersection count.
  const float back_side_intersection = self_ray_map.y;
  // From top to current voxel face intersection count.
  const float top_side_intersection = self_ray_map.z;
  // From current to left voxel face intersection count.
  const float left_side_intersection = _ray_map[int3(0, id.y, id.z)].x - self_ray_map.x;
  // From current to front voxel face intersection count.
  const float front_side_intersection = _ray_map[int3(id.x, 0, id.z)].y - self_ray_map.y;
  // From current to bottom voxel face intersection count.
  const float bottom_side_intersection = _ray_map[int3(id.x, id.y, 0)].z - self_ray_map.z;
  // Calculate the sign of the voxel.
  _sign_map_rw[id.xyz] =
    right_side_intersection - left_side_intersection +
    back_side_intersection - front_side_intersection +
    top_side_intersection - bottom_side_intersection;
}