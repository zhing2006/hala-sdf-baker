#include "../baker/sdf_baker.hlsl"

struct PushConstants {
  float threshold;
  float offset;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 1)]]
StructuredBuffer<Triangle> _triangles_uvw;

[[vk::binding(1, 1)]]
StructuredBuffer<uint> _triangles_in_voxels;

[[vk::binding(2, 1)]]
StructuredBuffer<uint> _accum_counters_buffer;

[[vk::binding(3, 1)]]
Texture3D<float> _sign_map;

[[vk::binding(4, 1)]]
Texture3D<float4> _voxels_texture;

[[vk::binding(5, 1)]]
RWStructuredBuffer<float4> _voxels_buffer_rw;

[[vk::binding(6, 1)]]
RWTexture3D<float> _distance_texture_rw;

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

[numthreads(8, 8, 8)]
void main(uint3 id : SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z)
    return;

  // Retrieve the seed coordinate from a 3D texture, normalized by the maximum dimension.
  const float3 seed_coord = _voxels_texture[int3(id.x, id.y, id.z)].xyz;
  const float3 voxel_coord = (float3(id.xyz) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension;

  // Determine the sign of the distance based on a threshold comparison.
  float sign_d = _sign_map[id.xyz] > g_push_constants.threshold ? -1 : 1;

  // Convert the seed coordinate back to integer indices.
  const int3 id_seed = int3(seed_coord * _max_dimension);

  // Get the start and end index for triangle iteration.
  uint start_triangle_id = 0;
  [branch]
  if(id3(id_seed) > 0) {
    start_triangle_id = _accum_counters_buffer[id3(id_seed) - 1];
  }
  uint end_triangle_id = _accum_counters_buffer[id3(id_seed)];

  float dist = 1e6f;
  for (uint i = start_triangle_id; (i < end_triangle_id) && (i < _upper_bound_count - 1); i++) {
    const uint triangle_index = _triangles_in_voxels[i];
    Triangle tri = _triangles_uvw[triangle_index];
    dist = min(dist, point_distance_to_triangle(voxel_coord, tri));
  }
  if (1e6f - dist < COMMON_EPS) {
    dist = length(seed_coord - voxel_coord);
  }
  dist = sign_d * dist - g_push_constants.offset;

  _voxels_buffer_rw[id3(id.xyz)] = float4(dist, dist, dist, dist);
  _distance_texture_rw[id.xyz] = dist;
}
