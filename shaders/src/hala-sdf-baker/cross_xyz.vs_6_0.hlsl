#include "cross_xyz.hlsl"

ToFragment main(uint vertex_id: SV_VertexID) {
  ToFragment output = (ToFragment)0;

  const float3 center = float3(g_push_constants.center[0], g_push_constants.center[1], g_push_constants.center[2]);
  const float3 extents = float3(g_push_constants.extents[0], g_push_constants.extents[1], g_push_constants.extents[2]);

  float3 vertex = g_vertices[vertex_id];
  output.uvw = (vertex + 1.0) * 0.5;
  vertex = vertex * extents + center;
  output.position = mul(g_push_constants.mvp, float4(vertex, 1.0));

  return output;
}