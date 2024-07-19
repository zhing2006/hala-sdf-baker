#include "udf_baker.hlsl"

[[vk::binding(0, 1)]]
RWTexture3D<uint> _distance_texture_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }

  // if (id.x == 0 && id.y == 0 && id.z == 0) {
  //   printf("dimensions: %d %d %d\n", _dimensions.x, _dimensions.y, _dimensions.z);
  //   printf("num_of_voxels: %d\n", _num_of_voxels);
  //   printf("num_of_triangles: %d\n", _num_of_triangles);
  //   printf("max_distance: %f\n", _max_distance);
  //   printf("initial_distance: %f\n", _initial_distance);
  //   printf("voxel_size: %f\n", _voxel_size);
  //   printf("min_bounds_extended: %f %f %f\n", _min_bounds_extended.x, _min_bounds_extended.y, _min_bounds_extended.z);
  //   printf("max_bounds_extended: %f %f %f\n", _max_bounds_extended.x, _max_bounds_extended.y, _max_bounds_extended.z);
  // }

  _distance_texture_rw[int3(id.x, id.y, id.z)] = float_flip(_initial_distance);
}