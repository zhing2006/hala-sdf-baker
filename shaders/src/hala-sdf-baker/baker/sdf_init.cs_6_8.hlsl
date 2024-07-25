#include "sdf_baker.hlsl"

[[vk::binding(0, 1)]]
RWStructuredBuffer<float4> _voxels_buffer_rw;

[[vk::binding(1, 1)]]
RWStructuredBuffer<uint> _counter_buffer_rw;

[[vk::binding(2, 1)]]
RWStructuredBuffer<uint> _accum_counter_buffer_rw;

[[vk::binding(3, 1)]]
RWTexture3D<float4> _ray_map_rw;

[[vk::binding(4, 1)]]
RWTexture3D<uint> _sign_map_rw;

[[vk::binding(5, 1)]]
RWTexture3D<uint> _sign_map_bis_rw;

[[vk::binding(6, 1)]]
RWTexture3D<float4> _voxels_texture_rw;

[[vk::binding(7, 1)]]
RWTexture3D<float4> _voxels_texture_bis_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }

  // if (id.x == 0 && id.y == 0 && id.z == 0) {
  //   printf("_dimensions: %d %d %d\n", _dimensions.x, _dimensions.y, _dimensions.z);
  //   printf("_max_dimension: %d\n", _max_dimension);
  //   printf("_upper_bound_count: %d\n", _upper_bound_count);
  //   printf("_num_of_triangles: %d\n", _num_of_triangles);
  //   printf("_max_size: %f\n", _max_size);
  //   printf("_min_bounds_extended: %f %f %f\n", _min_bounds_extended.x, _min_bounds_extended.y, _min_bounds_extended.z);
  //   printf("_max_bounds_extended: %f %f %f\n", _max_bounds_extended.x, _max_bounds_extended.y, _max_bounds_extended.z);
  // }

  _ray_map_rw[int3(id.x, id.y, id.z)] = float4(0, 0, 0, 0);
  _sign_map_rw[int3(id.x, id.y, id.z)] = 0;
  _sign_map_bis_rw[int3(id.x, id.y, id.z)] = 0;
  _voxels_texture_rw[int3(id.x, id.y, id.z)] = float4(0, 0, 0, 0);
  _voxels_texture_bis_rw[int3(id.x, id.y, id.z)] = float4(0, 0, 0, 0);

  _voxels_buffer_rw[id3(id)] = float4(0, 0, 0, 0);
  _counter_buffer_rw[id3(id)] = 0u;
  _accum_counter_buffer_rw[id3(id)] = 0u;
}