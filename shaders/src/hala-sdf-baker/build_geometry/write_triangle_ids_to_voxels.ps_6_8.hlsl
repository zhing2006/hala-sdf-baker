#include "../baker/sdf_baker.hlsl"
#include "draw.hlsl"

struct FragmentOutput {
  [[vk::location(0)]] float4 color: SV_Target0;
};

FragmentOutput main(ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

#ifdef USE_CONSERVATIVE_RASTERIZATION
  int3 depth_step, voxel_coord;
  bool can_step_backward, can_step_forward;
  get_voxel_coordinates(input.position, input.triangle_id, voxel_coord, depth_step, can_step_backward, can_step_forward);
#else
  int3 voxel_coord;
  get_voxel_coordinates(input.position, input.triangle_id, voxel_coord);
#endif

  uint index = 0u;
  InterlockedAdd(_counter_buffer_rw[id3(voxel_coord)], 1u, index);
  if (index < _upper_bound_count)
  _triangle_ids_buffer_rw[index] = input.triangle_id;
#ifdef USE_CONSERVATIVE_RASTERIZATION
  if (can_step_forward) {
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord + depth_step)], 1u, index);
    if (index < _upper_bound_count)
    _triangle_ids_buffer_rw[index] = input.triangle_id;
  }
  if (can_step_backward) {
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord - depth_step)], 1u, index);
    if (index < _upper_bound_count)
    _triangle_ids_buffer_rw[index] = input.triangle_id;
  }
#endif

  output.color = input.position;
  return output;
}