#include "common.hlsl"
#include "test_common.hlsl"

static const float3 positions[3] = {
  float3( 0.0, -1.0, 0.0),
  float3( 1.0,  1.0, 0.0),
  float3(-1.0,  1.0, 0.0)
};

[outputtopology("triangle")]
[numthreads(1, 1, 1)]
void main(
  out indices uint3 triangles[1],
  out vertices to_ps vertices[3],
  uint3 dispatchThreadID : SV_DispatchThreadID
) {
  const float3 offset = float3(0.0f, 0.0f, (float)dispatchThreadID);
  const ObjectUniformBuffer per_object_data = g_per_object_data[g_push_constants.object_index];

  SetMeshOutputCounts(3, 1);
  for (uint i = 0; i < 3; i++) {
    vertices[i].position = mul(per_object_data.mvp_mtx, float4(positions[i] + offset, 1.0));
    vertices[i].uv = float2(0.0f, 0.0f);
    vertices[i].normal = float3(0.0f, 0.0f, 1.0f);
    vertices[i].tangent = float3(1.0f, 0.0f, 0.0f);
  }

  SetMeshOutputCounts(3, 1);
  triangles[0] = uint3(0, 1, 2);
}