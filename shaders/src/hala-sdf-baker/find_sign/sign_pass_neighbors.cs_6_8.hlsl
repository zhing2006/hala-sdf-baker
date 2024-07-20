#include "../common/random.hlsl"
#include "../baker/sdf_baker.hlsl"

struct PushConstants {
  float normalize_factor;
  uint num_of_neighbors;
  uint pass_id;
  bool need_normalize;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 1)]]
Texture3D<float4> _ray_map;

[[vk::binding(1, 1)]]
Texture3D<float> _sign_map;

[[vk::binding(2, 1)]]
RWTexture3D<float> _sign_map_rw;

int3 generate_random_neighbor_offset(int neighbour_index, float max_dimension, float dist_to_surface) {
  // Uniformly distributed value between -1 and 1.
  const float r = 2.0f * generate_hashed_random_float(neighbour_index) - 1;
  // Random angle for the spherical coordinates.
  const float phi = 2.0f * PI * generate_hashed_random_float(neighbour_index + 1);
  // Radius factor based on cube root of uniform distribution, scaled by surface distance and dimension.
  const float radius = pow(generate_hashed_random_float(neighbour_index + 2), 1.0f / 3.0f) * max(1.0f, max_dimension * dist_to_surface);

  // Computes coordinates on the unit sphere.
  const float cos_theta = sqrt(1 - r * r);
  float sin_phi, cos_phi;
  sincos(phi, sin_phi, cos_phi);

  const float x = radius * cos_phi * cos_theta;
  const float y = radius * sin_phi * cos_theta;
  const float z = radius * r;

  return int3(x, y, z);
}

[numthreads(4, 4, 4)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z)
    return;

  const float4 self_ray_map = _ray_map[id.xyz];
  for (uint i = 0; i < g_push_constants.num_of_neighbors; i++) {
    int3 neighbors_offset = generate_random_neighbor_offset((i * g_push_constants.num_of_neighbors) + g_push_constants.pass_id, _max_dimension, 0.05f);
    int3 neighbors_index;
    neighbors_index.x = min((int)(_dimensions.x - 1), max(0, (int)id.x + neighbors_offset.x));
    neighbors_index.y = min((int)(_dimensions.y - 1), max(0, (int)id.y + neighbors_offset.y));
    neighbors_index.z = min((int)(_dimensions.z - 1), max(0, (int)id.z + neighbors_offset.z));

    float accum_sign = 0.0f;
    // xyz
    accum_sign += (self_ray_map.x - _ray_map[int3(neighbors_index.x, id.y, id.z)].x);
    accum_sign += (_ray_map[int3(neighbors_index.x, id.y, id.z)].y - _ray_map[int3(neighbors_index.x, neighbors_index.y, id.z)].y);
    accum_sign += (_ray_map[int3(neighbors_index.x, neighbors_index.y, id.z)].z - _ray_map[neighbors_index].z);

    // xzy
    accum_sign += (self_ray_map.x - _ray_map[int3(neighbors_index.x, id.y, id.z)].x);
    accum_sign += (_ray_map[int3(neighbors_index.x, id.y, id.z)].z - _ray_map[int3(neighbors_index.x, id.y, neighbors_index.z)].z);
    accum_sign += (_ray_map[int3(neighbors_index.x, id.y, neighbors_index.z)].y - _ray_map[neighbors_index].y);

    // yxz
    accum_sign += (self_ray_map.y - _ray_map[int3(id.x, neighbors_index.y, id.z)].y);
    accum_sign += (_ray_map[int3(id.x, neighbors_index.y, id.z)].x - _ray_map[int3(neighbors_index.x, neighbors_index.y, id.z)].x);
    accum_sign += (_ray_map[int3(neighbors_index.x, neighbors_index.y, id.z)].z - _ray_map[neighbors_index].z);

    // yzx
    accum_sign += (self_ray_map.y - _ray_map[int3(id.x, neighbors_index.y, id.z)].y);
    accum_sign += (_ray_map[int3(id.x, neighbors_index.y, id.z)].z - _ray_map[int3(id.x, neighbors_index.y, neighbors_index.z)].z);
    accum_sign += (_ray_map[int3(id.x, neighbors_index.y, neighbors_index.z)].x - _ray_map[neighbors_index].x);

    // zyx
    accum_sign += (self_ray_map.z - _ray_map[int3(id.x, id.y, neighbors_index.z)].z);
    accum_sign += (_ray_map[int3(id.x, id.y, neighbors_index.z)].y - _ray_map[int3(id.x, neighbors_index.y, neighbors_index.z)].y);
    accum_sign += (_ray_map[int3(id.x, neighbors_index.y, neighbors_index.z)].x - _ray_map[neighbors_index].x);

    // zxy
    accum_sign += (self_ray_map.z - _ray_map[int3(id.x, id.y, neighbors_index.z)].z);
    accum_sign += (_ray_map[int3(id.x, id.y, neighbors_index.z)].x - _ray_map[int3(neighbors_index.x, id.y, neighbors_index.z)].x);
    accum_sign += (_ray_map[int3(neighbors_index.x, id.y, neighbors_index.z)].y - _ray_map[neighbors_index].y);

    _sign_map_rw[id.xyz] += g_push_constants.normalize_factor * accum_sign + 6 * _sign_map[neighbors_index];
  }

  if (g_push_constants.need_normalize) {
    const float normalize_factor_final = g_push_constants.normalize_factor + g_push_constants.num_of_neighbors * 6 * g_push_constants.normalize_factor;
    _sign_map_rw[id.xyz] /= normalize_factor_final;
  }
}