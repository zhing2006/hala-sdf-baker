#include "../baker/udf_baker.hlsl"
#include "mesh.hlsl"

#define GRID_MARGIN int3(1, 1, 1)
#define EPSILON 1e-6f

[[vk::binding(3, 1)]]
RWTexture3D<uint> _distance_texture_rw;

float distance_point_2_edge(float3 p, float3 x0, float3 x1) {
  if (x0.x > x1.x) {
    const float3 temp = x0;
    x0 = x1;
    x1 = temp;
  }

  const float3 edge = x1 - x0;
  const float t = saturate(dot(x1 - p, edge) / dot(edge, edge));
  const float3 projection = x0 * t + x1 * (1.0f - t);
  const float3 diff = p - projection;
  const float len = length(diff);
  if (len < EPSILON) {
    return 0;
  } else {
    return len;
  }
}

float distance_point_2_triangle(float3 p, float3 x0, float3 x1, float3 x2) {
  // Calculate the normal vector of the triangle.
  const float3 edge1 = x1 - x0;
  const float3 edge2 = x2 - x0;
  float3 normal = cross(edge1, edge2);
  const float normal_length = length(normal);

  if (normal_length < EPSILON) {
    // The triangle is degenerate, return the distance to the nearest vertex.
    const float d0 = length(p - x0);
    const float d1 = length(p - x1);
    const float d2 = length(p - x2);
    return min(d0, min(d1, d2));
  } else {
    normal = normal / normal_length;

    // Calculate the signed distance from the point to the triangle plane.
    const float distance_2_plane = dot(normal, p - x0);

    // Project the point onto the triangle plane.
    const float3 projected_point = p - distance_2_plane * normal;

    // Check if the projected point is inside the triangle.
    const float3 c0 = cross(x1 - x0, projected_point - x0);
    const float3 c1 = cross(x2 - x1, projected_point - x1);
    const float3 c2 = cross(x0 - x2, projected_point - x2);

    if (dot(normal, c0) >= 0 && dot(normal, c1) >= 0 && dot(normal, c2) >= 0) {
      // Projected point is inside the triangle, return the signed or unsigned distance to the plane.
#ifdef SIGNED
      return distance_2_plane;
#else
      return abs(distance_2_plane);
#endif
    } else {
      // Projected point is outside the triangle, return the signed or unsigned distance to the nearest edge.
      const float d0 = distance_point_2_edge(p, x0, x1);
      const float d1 = distance_point_2_edge(p, x1, x2);
      const float d2 = distance_point_2_edge(p, x0, x2);

      float min_distance = min(d0, min(d1, d2));

#ifdef SIGNED
      min_distance = (distance_2_plane < 0) ? -min_distance : min_distance;
#endif

      return min_distance;
    }
  }
}

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
        float distance = distance_point_2_triangle(voxel_position, vertex0, vertex1, vertex2);
        uint distance_as_uint = float_flip(distance);
        InterlockedMin(_distance_texture_rw[voxel_coord], distance_as_uint);
      }
    }
  }
}