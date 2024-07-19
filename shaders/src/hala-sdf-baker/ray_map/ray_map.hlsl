struct PushConstants {
  uint3 ray_map_offset;
};

[[vk::push_constant]]
PushConstants g_push_constants;

float intersect_segment_triangle(float3 segment_start, float3 segment_end, Triangle tri, out float t_value) {
  /*
   * Triangle plane equation: n * (P - A) = 0
   * Segment equation: P(t) = Q + t(S - Q)
   * n dot ((Q + t(S - Q)) - A) = 0
   * n dot (Q - A + t(S - Q)) = 0
   * n dot (Q - A) + t(n dot (S - Q)) = 0
   * 𝑣 = 𝑄 - 𝐴, 𝑑 = 𝑆 − 𝑄
   * t = - (n dot 𝑣) / (n dot d)
   */

  // Calculate edge vectors of the triangle.
  const float3 edge1 = tri.b - tri.a;
  const float3 edge2 = tri.c - tri.a;
  const float3 end_to_start = segment_start - segment_end; // Q - S = -d

  // Compute the normal vector of the triangle.
  const float3 normal = cross(edge1, edge2);
  const float dot_product = dot(end_to_start, normal);
  const float inverse_dot_product = 1.0f / dot_product;

  const float side = sign(dot_product);
  // Calculate the intersection t value of the segment with the plane of the triangle.
  const float3 vertex0_to_start = segment_start - tri.a; // Q - A = v
  float t = dot(vertex0_to_start, normal) * inverse_dot_product; // t = - (n dot v) / (n dot d) = (n dot v) / (n dot -d)
  if (t < -INTERSECT_EPS || t > 1 + INTERSECT_EPS) {
    t_value = 1e10;
    return 0; // The intersection is outside the segment.
  } else {
    // Calculate barycentric coordinates and check if they are within bounds.
    const float3 cross_product = cross(end_to_start, vertex0_to_start);
    const float u = dot(edge2, cross_product) * inverse_dot_product;
    const float v = -dot(edge1, cross_product) * inverse_dot_product;
    float edge_coefficient = 1.0f;

    if (u < -BARY_EPS || u > 1 + BARY_EPS || v < -BARY_EPS || u + v > 1 + BARY_EPS) {
      t_value = 1e10;
      return 0; // The intersection is outside the triangle.
    } else {
      const float w = 1.0f - u - v;
      if (abs(u) < BARY_EPS || abs(v) < BARY_EPS || abs(w) < BARY_EPS) {
        edge_coefficient = 0.5f; // 0.5 the intersection is on an edge.
      }

      t_value = t; // Write t_value only if all the other tests passed.
      return side * edge_coefficient;  // Return the intersection result with edge coefficient adjustment
    }
  }
}

void test_intersection_6_rays(
  in Triangle tri,              // Input triangle used for the intersection test.
  in int3 voxel_id,             // The voxel coordinates from where rays are shot.
  out float3 intersect_forward, // Outputs the number of intersections in forward directions for x, y, z axes.
  out float3 intersect_backward // Outputs the number of intersections in backward directions for x, y, z axes.
) {
  // Initialize intersection counts to zero.
  intersect_forward = float3(0.0f, 0.0f, 0.0f);
  intersect_backward = float3(0.0f, 0.0f, 0.0f);

  // Intersection parameters to store the intersection points along the ray.
  float t = 1e10f;
  // Start and end points of the ray in normalized coordinates.
  float3 p, q;
  // Temporary variable to accumulate the intersection direction sign.
  float intersect = 0;

  // Test x-direction rays from voxel center to the adjacent voxel center.
  p = (float3(voxel_id) + float3(0.0f, 0.5f, 0.5f)) / _max_dimension;
  q = (float3(voxel_id) + float3(1.0f, 0.5f, 0.5f)) / _max_dimension;
  // The line is from -x to +x, so the triangle is facing left.
  // Negative intersection means the triangle is facing -x direction.
  intersect = -intersect_segment_triangle(p, q, tri, t);
  if (t < 0.5f) {
    // If the intersection is on the left side, it is backward.
    intersect_backward.x += float(intersect);
  } else {
    // If the intersection is on the right side, it is forward.
    intersect_forward.x += float(intersect);
  }

  // Test y-direction rays.
  p = (float3(voxel_id) + float3(0.5f, 0.0f, 0.5f)) / _max_dimension;
  q = (float3(voxel_id) + float3(0.5f, 1.0f, 0.5f)) / _max_dimension;
  intersect = -intersect_segment_triangle(p, q, tri, t);
  if (t < 0.5f) {
    intersect_backward.y += float(intersect);
  } else {
    intersect_forward.y += float(intersect);
  }

  // Test z-direction rays.
  p = (float3(voxel_id) + float3(0.5f, 0.5f, 0.0f)) / _max_dimension;
  q = (float3(voxel_id) + float3(0.5f, 0.5f, 1.0f)) / _max_dimension;
  intersect = -intersect_segment_triangle(p, q, tri, t);
  if (t < 0.5f) {
    intersect_backward.z += float(intersect);
  } else {
    intersect_forward.z += float(intersect);
  }
}