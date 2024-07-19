#include "rgb_triangle.hlsl"

ToFragment main(uint vertex_id: SV_VertexID) {
  ToFragment output = (ToFragment)0;

  const float2 vertices[3] = {
    float2( 0.0,  1.0),
    float2(-1.0, -1.0),
    float2( 1.0, -1.0)
  };
  const float3 colors[3] = {
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0, 0.0, 1.0)
  };

  output.position = float4(vertices[vertex_id], 0.0, 1.0);
  output.color = colors[vertex_id];

  return output;
}