struct Triangle {
  float3 a, b, c;
};

inline float dot2(float3 v) {
  return dot(v, v);
}

float point_distance_to_triangle(float3 pt, Triangle tri) {
  // Calculate edge vectors.
  const float3 edge_a_to_b = tri.b - tri.a;
  const float3 edge_b_to_c = tri.c - tri.b;
  const float3 edge_c_to_a = tri.a - tri.c;

  // Calculate vectors from point to triangle vertices.
  const float3 vector_p_to_a = pt - tri.a;
  const float3 vector_p_to_b = pt - tri.b;
  const float3 vector_p_to_c = pt - tri.c;

  // Calculate normal of the triangle
  const float3 normal = cross(edge_a_to_b, edge_c_to_a);

  // Calculate squared distance to the triangle
  const float dist_squared =
    // Inside/outside test using signs of dot products
    (sign(dot(cross(edge_a_to_b, normal), vector_p_to_a)) +
     sign(dot(cross(edge_b_to_c, normal), vector_p_to_b)) +
     sign(dot(cross(edge_c_to_a, normal), vector_p_to_c)) < 2.0f)
    ?
    // If outside, calculate distance to the nearest edge
    min(min(
      dot2(edge_a_to_b * clamp(dot(edge_a_to_b, vector_p_to_a) / dot2(edge_a_to_b), 0.0, 1.0) - vector_p_to_a),
      dot2(edge_b_to_c * clamp(dot(edge_b_to_c, vector_p_to_b) / dot2(edge_b_to_c), 0.0, 1.0) - vector_p_to_b)),
      dot2(edge_c_to_a * clamp(dot(edge_c_to_a, vector_p_to_c) / dot2(edge_c_to_a), 0.0, 1.0) - vector_p_to_c))
    :
    // If inside, calculate distance to the face
    dot(normal, vector_p_to_a) * dot(normal, vector_p_to_a) / dot2(normal);

  return sqrt(dist_squared);
}
