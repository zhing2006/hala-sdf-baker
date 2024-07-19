#include "../baker/baker.hlsl"
#include "mesh.hlsl"

[[vk::binding(3, 1)]]
RWStructuredBuffer<uint> _coord_flip_buffer_rw;

[numthreads(64, 1, 1)]
void main(uint3 id: SV_DispatchThreadID) {
  if (id.x >= _num_triangles) {
    return;
  }

  const float3 a = get_vertex_pos(id.x, 0);
  const float3 b = get_vertex_pos(id.x, 1);
  const float3 c = get_vertex_pos(id.x, 2);
  const float3 edge0 = b - a;
  const float3 edge1 = c - b;
  const float3 n = abs(cross(edge0, edge1));
  if (n.x > max(n.y, n.z) + 1e-6f) {  // Plus epsilon to make comparison more stable.
    // Triangle nearly parallel to YZ plane
    _coord_flip_buffer_rw[id.x] = 2;
  } else if (n.y > max(n.x, n.z) + 1e-6f) {
    // Triangle nearly parallel to ZX plane
    _coord_flip_buffer_rw[id.x] = 1;
  } else {
    // Triangle nearly parallel to XY plane
    _coord_flip_buffer_rw[id.x] = 0;
  }
}