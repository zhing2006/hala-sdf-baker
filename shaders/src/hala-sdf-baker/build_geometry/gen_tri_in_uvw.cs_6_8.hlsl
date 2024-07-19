#include "../baker/sdf_baker.hlsl"
#include "mesh.hlsl"

[[vk::binding(3, 1)]]
RWStructuredBuffer<Triangle> _triangles_uvw_rw;

[numthreads(64, 1, 1)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _num_of_triangles) {
    return;
  }

  // if (id.x == 0) {
  //   printf("_vertex_position_offset: %d\n", _vertex_position_offset);
  //   printf("_vertex_stride: %d\n", _vertex_stride);
  //   printf("_index_stride: %d\n", _index_stride);
  // }

  // Calculate UVW coordinates for each triangle in the box witch size is _MaxExtent.
  // For example, if the mesh bounds = {min = (0, 1, 2), max = (5, 4, 3)},
  // then the _MaxExtent = 5 and the box size = (2.5, 1.5, 0.5) and the center = (2.5, 2.5, 2.5).
  const float3 extents = 0.5 * (_max_bounds_extended - _min_bounds_extended);
  const float3 center = 0.5 * (_max_bounds_extended + _min_bounds_extended);
  // If the triangle vertex a = (3, 3, 2), then the UVW coordinates for this vertex will be (
  // U = (3 - 2.5 + 2.5) / 5 = 0.3,
  // V = (3 - 2.5 + 1.5) / 5 = 0.4,
  // W = (2 - 2.5 + 0.5) / 5 = 0.0).
  // Transform the UVW to the range [0, 1].
  Triangle tri_uvw;
  tri_uvw.a = (get_vertex_pos(id.x, 0) - center + extents) / _max_extent;
  tri_uvw.b = (get_vertex_pos(id.x, 1) - center + extents) / _max_extent;
  tri_uvw.c = (get_vertex_pos(id.x, 2) - center + extents) / _max_extent;

  _triangles_uvw_rw[id.x] = tri_uvw;
}