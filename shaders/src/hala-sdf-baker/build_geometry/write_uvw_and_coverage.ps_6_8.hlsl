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

  // Write the uvw(0 - 1) in the cube to the voxel buffer.
  // Also increment the counter buffer.
  float3 voxel_uvw = ((float3(voxel_coord) + float3(0.5f, 0.5f, 0.5f)) / max(_dimensions[0], max(_dimensions[1], _dimensions[2])));
  _voxels_buffer_rw[id3(voxel_coord)] = float4(voxel_uvw, 1.0f);
  InterlockedAdd(_counter_buffer_rw[id3(voxel_coord)], 1u);
#ifdef USE_CONSERVATIVE_RASTERIZATION
  if (can_step_forward) {
    _voxels_buffer_rw[id3(voxel_coord + depth_step)] = float4(voxel_uvw, 1.0f);
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord + depth_step)], 1u);
  }
  if (can_step_backward) {
    _voxels_buffer_rw[id3(voxel_coord - depth_step)] = float4(voxel_uvw, 1.0f);
    InterlockedAdd(_counter_buffer_rw[id3(voxel_coord - depth_step)], 1u);
  }
#endif

  output.color = float4(voxel_uvw, 1);
  return output;
}