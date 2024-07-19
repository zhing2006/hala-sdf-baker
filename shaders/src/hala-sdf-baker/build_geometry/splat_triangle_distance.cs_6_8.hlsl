#include "../baker/udf_baker.hlsl"
#include "mesh.hlsl"

[[vk::binding(3, 1)]]
RWTexture3D<uint> _distance_texture_rw;

[numthreads(64, 1, 1)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _num_of_triangles) {
    return;
  }

  if (id.x == 0) {
    printf("_vertex_position_offset: %d\n", _vertex_position_offset);
    printf("_vertex_stride: %d\n", _vertex_stride);
    printf("_index_stride: %d\n", _index_stride);
  }
}