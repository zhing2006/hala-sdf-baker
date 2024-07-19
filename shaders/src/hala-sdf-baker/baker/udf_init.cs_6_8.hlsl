#include "udf_baker.hlsl"

[[vk::binding(0, 1)]]
RWTexture3D<uint> _distance_texture_rw;

[numthreads(8, 8, 8)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _dimensions.x || id.y >= _dimensions.y || id.z >= _dimensions.z) {
    return;
  }

  if (id.x == 0 && id.y == 0 && id.z == 0) {
    printf("dimensions: %d %d %d\n", _dimensions.x, _dimensions.y, _dimensions.z);
    printf("num_of_voxels: %d\n", _num_of_voxels);
    printf("num_of_triangles: %d\n", _num_of_triangles);
    printf("max_distance: %f\n", _max_distance);
    printf("initial_distance: %f\n", _initial_distance);
  }

  _distance_texture_rw[int3(id.x, id.y, id.z)] = 0;
}