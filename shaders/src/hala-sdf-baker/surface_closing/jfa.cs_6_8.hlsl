#include "../baker/sdf_baker.hlsl"

struct PushConstants {
  int offset;
};

[[vk::push_constant]]
PushConstants g_push_constants;

[[vk::binding(0, 1)]]
RWTexture3D<float4> _voxels_texture;

[[vk::binding(1, 1)]]
RWTexture3D<float4> _voxels_texture_rw;

[numthreads(4,4,4)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z)
    return;

  float best_distance = 1e6f;
  float3 best_coord = float3(0.0f, 0.0f, 0.0f);
  [unroll(3)]
  for (int z = -1; z <= 1; z++) {
    [unroll(3)]
    for (int y = -1; y <= 1; y++) {
      [unroll(3)]
      for (int x = -1; x <= 1; x++) {
        int3 sample_coord;
        sample_coord.x = min((int)(_dimensions.x - 1), max(0, (int)id.x + x * g_push_constants.offset));
        sample_coord.y = min((int)(_dimensions.y - 1), max(0, (int)id.y + y * g_push_constants.offset));
        sample_coord.z = min((int)(_dimensions.z - 1), max(0, (int)id.z + z * g_push_constants.offset));

        float3 seed_coord = _voxels_texture[sample_coord].xyz;
        float dist = length(seed_coord - (float3(id.xyz) + float3(0.5f, 0.5f, 0.5f)) / _max_dimension);
        if ((seed_coord.x != 0.0f || seed_coord.y != 0.0f || seed_coord.z != 0.0f) && dist < best_distance) {
          best_coord = seed_coord;
          best_distance = dist;
        }
      }
    }
  }

  _voxels_texture_rw[id.xyz] = float4(best_coord, best_distance);
}