#include "../baker/udf_baker.hlsl"
#include "mesh.hlsl"

#define GRID_MARGIN int3(1, 1, 1)

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

  const float3 vertex0 = get_vertex_pos(id.x, 0);
  const float3 vertex1 = get_vertex_pos(id.x, 1);
  const float3 vertex2 = get_vertex_pos(id.x, 2);

  const float3 aabb_min = min(vertex0, min(vertex1, vertex2)) - _voxel_size;
  const float3 aabb_max = max(vertex0, max(vertex1, vertex2)) + _voxel_size;
  int3 voxel_min = get_voxel_coord(aabb_min);
  int3 voxel_max = get_voxel_coord(aabb_max) + GRID_MARGIN;
  voxel_min = max(0, min(voxel_min, _dimensions - 1));
  voxel_max = max(0, min(voxel_max, _dimensions - 1));

  for (int z = voxel_min.z; z <= voxel_max.z; ++z) {
    for (int y = voxel_min.y; y <= voxel_max.y; ++y) {
      for (int x = voxel_min.x; x <= voxel_max.x; ++x) {
        int3 voxel_coord = int3(x, y, z);
        uint id = id3(voxel_coord);
        float3 voxel_position = get_position(voxel_coord);
        float distance = _max_distance;
        uint distance_as_uint = float_flip(distance);
        InterlockedMin(_distance_texture_rw[voxel_coord], distance_as_uint);
      }
    }
  }
}