#include "../baker/sdf_baker.hlsl"
#include "ray_map.hlsl"

[[vk::binding(0, 1)]]
StructuredBuffer<uint> _accum_counter_buffer;

[[vk::binding(1, 1)]]
StructuredBuffer<uint> _triangles_in_voxels;

[[vk::binding(2, 1)]]
StructuredBuffer<Triangle> _triangles_uvw;

[[vk::binding(3, 1)]]
RWTexture3D<float4> _ray_map_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  id = 2 * id + g_push_constants.ray_map_offset;
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z)
    return;

  uint start_triangle_id = 0;
  [branch]
  if (id3(id) > 0) {
    start_triangle_id = _accum_counter_buffer[id3(id) - 1];
  }
  uint end_triangle_id = _accum_counter_buffer[id3(id)];

  for (uint i = start_triangle_id; i < end_triangle_id && (i < _upper_bound_count - 1); i++) {
    const uint triangle_index = _triangles_in_voxels[i];
    const Triangle tri = _triangles_uvw[triangle_index];
    float3 intersect_forward, intersect_backward;
    test_intersection_3_rays(tri, int3(id.xyz), intersect_forward, intersect_backward);

    // Save the forward intersection count(-axis is negative, +axis is positive) to the current voxel.
    _ray_map_rw[id.xyz] += float4(intersect_forward, 1.0f);

    // Save the x direction backward intersection count(-axis is negative, +axis is positive) to the left voxel.
    if (id.x > 0) {
      _ray_map_rw[int3(id.x - 1, id.y, id.z)] += float4(intersect_backward.x, 0.0f, 0.0f, 1.0f);
    }

    // Save the y direction backward intersection count(-axis is negative, +axis is positive) to the bottom voxel.
    if (id.y > 0) {
      _ray_map_rw[int3(id.x, id.y - 1, id.z)] += float4(0.0f, intersect_backward.y, 0.0f, 1.0f);
    }

    // Save the z direction backward intersection count(-axis is negative, +axis is positive) to the front voxel.
    if (id.z > 0) {
      _ray_map_rw[int3(id.x, id.y, id.z - 1)] += float4(0.0f, 0.0f, intersect_backward.z, 1.0f);
    }
  }
}