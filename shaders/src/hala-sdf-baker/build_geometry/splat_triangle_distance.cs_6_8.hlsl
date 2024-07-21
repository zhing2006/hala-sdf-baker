#include "../baker/udf_baker.hlsl"
#include "mesh.hlsl"

#define GRID_MARGIN int3(1, 1, 1)
#define EPSILON 1e-6f

[[vk::binding(3, 1)]]
RWTexture3D<uint> _distance_texture_rw;

[numthreads(64, 1, 1)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _num_of_triangles) {
    return;
  }

//   if (id.x == 0) {
//     printf("_vertex_position_offset: %d\n", _vertex_position_offset);
//     printf("_vertex_stride: %d\n", _vertex_stride);
//     printf("_index_stride: %d\n", _index_stride);
//   }

  const float3 extents = 0.5 * (_max_bounds_extended - _min_bounds_extended);
  const float3 center = 0.5 * (_max_bounds_extended + _min_bounds_extended);

  Triangle tri_uvw;
  tri_uvw.a = (get_vertex_pos(id.x, 0) - center + extents) / _max_size;
  tri_uvw.b = (get_vertex_pos(id.x, 1) - center + extents) / _max_size;
  tri_uvw.c = (get_vertex_pos(id.x, 2) - center + extents) / _max_size;

  const float3 aabb_min = min(tri_uvw.a, min(tri_uvw.b, tri_uvw.c));
  const float3 aabb_max = max(tri_uvw.a, max(tri_uvw.b, tri_uvw.c));
  int3 voxel_min = int3(aabb_min * _max_dimension) - GRID_MARGIN;
  int3 voxel_max = int3(aabb_max * _max_dimension) + GRID_MARGIN;
  voxel_min = max(0, min(voxel_min, int3(_dimensions) - 1));
  voxel_max = max(0, min(voxel_max, int3(_dimensions) - 1));

  for (int z = voxel_min.z; z <= voxel_max.z; ++z) {
    for (int y = voxel_min.y; y <= voxel_max.y; ++y) {
      for (int x = voxel_min.x; x <= voxel_max.x; ++x) {
        const float3 voxel_coord = (float3(x, y, z) + float3(0.5, 0.5, 0.5)) / _max_dimension;
        float distance = point_distance_to_triangle(voxel_coord, tri_uvw);
        uint distance_as_uint = float_flip(distance);
        InterlockedMin(_distance_texture_rw[int3(x, y, z)], distance_as_uint);
      }
    }
  }
}