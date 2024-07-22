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
  //   printf("initial_distance: %f\n", _initial_distance);
  //   printf("max_size: %f\n", _max_size);
  //   printf("max_dimension: %d\n", _max_dimension);
  //   printf("center: %f %f %f\n", _center.x, _center.y, _center.z);
  //   printf("extents: %f %f %f\n", _extents.x, _extents.y, _extents.z);
  // }

  _distance_texture_rw[int3(id.x, id.y, id.z)] = float_flip(_initial_distance);
}