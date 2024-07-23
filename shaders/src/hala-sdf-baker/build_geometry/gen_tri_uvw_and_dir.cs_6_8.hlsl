#include "../baker/sdf_baker.hlsl"
#include "mesh.hlsl"

[[vk::binding(3, 1)]]
RWStructuredBuffer<uint> _coord_flip_buffer_rw;

[[vk::binding(4, 1)]]
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

  const float3 a = get_vertex_pos(id.x, 0);
  const float3 b = get_vertex_pos(id.x, 1);
  const float3 c = get_vertex_pos(id.x, 2);
  const float3 edge0 = b - a;
  const float3 edge1 = c - b;
  const float3 n = abs(cross(edge0, edge1));
  if (n.x > max(n.y, n.z) + 1e-6f) {  // Plus epsilon to make comparison more stable.
    // Triangle nearly parallel to YZ plane
    _coord_flip_buffer_rw[id.x] = 2;
  } else if (n.y > max(n.x, n.z) + 1e-6f) {
    // Triangle nearly parallel to ZX plane
    _coord_flip_buffer_rw[id.x] = 1;
  } else {
    // Triangle nearly parallel to XY plane
    _coord_flip_buffer_rw[id.x] = 0;
  }

  // Calculate UVW coordinates for each triangle in the box witch size is _MaxExtent.
  // For example, if the mesh bounds = {min = (0, 1, 2), max = (5, 4, 3)},
  // then the _MaxExtent = 5 and the box size = (2.5, 1.5, 0.5) and the center = (2.5, 2.5, 2.5).
  // If the triangle vertex a = (3, 3, 2), then the UVW coordinates for this vertex will be (
  // U = (3 - 2.5 + 2.5) / 5 = 0.3,
  // V = (3 - 2.5 + 1.5) / 5 = 0.4,
  // W = (2 - 2.5 + 0.5) / 5 = 0.0).
  // Transform the UVW to the range [0, 1].

  Triangle tri_uvw;
  tri_uvw.a = (a - _center + _extents) / _max_size;
  tri_uvw.b = (b - _center + _extents) / _max_size;
  tri_uvw.c = (c - _center + _extents) / _max_size;

  _triangles_uvw_rw[id.x] = tri_uvw;
}