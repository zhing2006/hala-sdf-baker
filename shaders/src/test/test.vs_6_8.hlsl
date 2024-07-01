#include "common.hlsl"
#include "test_common.hlsl"

struct vs_in {
  [[vk::location(0)]] float4 position: POSITION;
  [[vk::location(1)]] float4 normal: NORMAL;
  [[vk::location(2)]] float4 tangent: TANGENT;
  [[vk::location(3)]] float4 uv: TEXCOORD0;
};

to_ps main(vs_in input) {
  to_ps output = (to_ps)0;

  const ObjectUniformBuffer per_object_data = g_per_object_data[g_push_constants.object_index];

  output.position = mul(per_object_data.mvp_mtx, float4(input.position.xyz, 1.0));
  output.uv = input.uv.xy;
  output.normal = normalize(mul(float4(input.normal.xyz, 0.0), per_object_data.i_m_mtx).xyz);
  output.tangent = normalize(mul(float4(input.tangent.xyz, 0.0), per_object_data.i_m_mtx).xyz);

  // printf("normal: %f %f %f\n", normal.x, normal.y, normal.z);

  return output;
}