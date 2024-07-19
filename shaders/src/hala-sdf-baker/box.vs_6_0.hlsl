#include "box.hlsl"

ToFragment main(uint vertex_id: SV_VertexID, uint instance_id : SV_InstanceID) {
  ToFragment output = (ToFragment)0;

  const float3 center = float3(g_push_constants.center[0], g_push_constants.center[1], g_push_constants.center[2]);
  const float3 extents = float3(g_push_constants.extents[0], g_push_constants.extents[1], g_push_constants.extents[2]);

  float3 vertex = g_vertices[instance_id * 2 + vertex_id];
  vertex = vertex * extents + center;
  output.position = mul(g_push_constants.mvp, float4(vertex, 1.0));

  output.color = float4(g_push_constants.color[0], g_push_constants.color[1], g_push_constants.color[2], g_push_constants.color[3]);

  return output;
}