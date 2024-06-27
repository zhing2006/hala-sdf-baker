#include "common.hlsl"
#include "test_common.hlsl"

struct vs_in {
  [[vk::location(0)]] float4 position: POSITION;
  [[vk::location(1)]] float4 normal: NORMAL;
  [[vk::location(2)]] float4 tangent: TANGENT;
  [[vk::location(3)]] float4 uv: TEXCOORD0;
};

vs_to_ps main(vs_in input) {
  vs_to_ps output = (vs_to_ps)0;

  const float4x4 mvp_mtx = g_per_object_data[g_push_constants.object_index].mvp_mtx;
  const float4x4 i_m_mtx = g_per_object_data[g_push_constants.object_index].i_m_mtx;

  output.position = mul(mvp_mtx, float4(input.position.xyz, 1.0));
  output.uv = input.uv.xy;
  const float3 normal = normalize(mul(float4(input.normal.xyz, 0.0), i_m_mtx).xyz);
  output.color = normal.xyz * 0.5 + 0.5;

  // printf("normal: %f %f %f\n", normal.x, normal.y, normal.z);

  return output;
}